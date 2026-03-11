use crate::budget::BudgetError;
use crate::models::config::Config;
use crate::models::csv::tables::{Categories, Plans, Report, Savings};
use crate::storage::{BasicRepository, FileStorage, Repository, StorageError};
use std::fs;
use std::{error::Error, path::Path, path::PathBuf};
use tracing::log::{debug, error};

#[derive(Debug, Clone)]
pub struct Account {
    pub config: Config,
    pub root_path: PathBuf,
}

impl Account {
    const DB_ROOT_DIRNAME: &'static str = "tinyexpenses";
    const CONFIG_FILENAME: &'static str = "config.toml";
    const EXPENSES_FILENAME: &'static str = "expenses.csv"; // TODO: Should we replace with report.csv? 
    const PLANS_FILENAME: &'static str = "plans.csv";
    const CATEGORIES_FILENAME: &'static str = "categories.csv";
    const SAVINGS_FILENAME: &'static str = "savings.csv";

    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let root_path = path.join(Self::DB_ROOT_DIRNAME);
        let config_path = path.join(Self::CONFIG_FILENAME);
        let config = Config::from(&config_path).and_read()?;

        let account = Account {
            config: config,
            root_path: root_path,
        };

        if !account.root_path.exists() {
            fs::create_dir(&account.root_path).map_err(|err| {
                format!(
                    "Unable to create '{}' directory ({}) as '{}' does not exists.",
                    Self::DB_ROOT_DIRNAME,
                    err,
                    account.root_path.to_str().unwrap_or_default()
                )
            })?;
        }

        Ok(account)
    }

    pub fn get_plans(&self, year: i32) -> Result<Plans, Box<dyn Error>> {
        let plans_path = self.db_path_from_year(year, Self::PLANS_FILENAME);
        let mut plans = Plans::from(&plans_path);

        if plans.exists() {
            plans.read()?;
            Ok(plans)
        } else {
            Err(BudgetError::Plans(StorageError::FileNotExists.into()).into())
        }
    }
    pub fn get_report(&self, year: i32) -> Result<Report, Box<dyn Error>> {
        let report_path = self.db_path_from_year(year, Self::EXPENSES_FILENAME);
        let mut report = Report::from(&report_path);

        if report.exists() {
            report.read()?;
            Ok(report)
        } else {
            Err(BudgetError::Report(StorageError::FileNotExists.into()).into())
        }
    }

    pub fn get_categories(&self, year: i32) -> Result<Categories, Box<dyn Error>> {
        let categories_path = self.db_path_from_year(year, Self::CATEGORIES_FILENAME);
        let mut categories = Categories::from(&categories_path);

        if categories.exists() {
            categories.read()?;
            Ok(categories)
        } else {
            Err(BudgetError::Categories(StorageError::FileNotExists.into()).into())
        }
    }

    pub fn get_savings(&self) -> Savings {
        let savings_path = self.db_path_from_root(Self::SAVINGS_FILENAME);
        let mut savings = Savings::from(&savings_path);

        if !savings.exists() {
            if let Err(err) = savings.create() {
                error!("Unable to create savings.");
                debug!("{}", err);
                panic!("Unable to create savings.");
            }
        }

        if let Err(err) = savings.read() {
            error!("Unable to read savings.");
            debug!("{}", err);
            panic!("Unable to read savings.");
        };

        savings
    }

    pub fn create_plans(&self, year: i32) -> Result<(), StorageError> {
        self.create_storage_in_year(year, Self::PLANS_FILENAME)?;
        Ok(())
    }

    pub fn create_report(&self, year: i32) -> Result<(), StorageError> {
        self.create_storage_in_year(year, Self::EXPENSES_FILENAME)?;
        Ok(())
    }

    pub fn create_categories(&self, year: i32) -> Result<(), StorageError> {
        self.create_storage_in_year(year, Self::CATEGORIES_FILENAME)?;
        Ok(())
    }

    pub fn copy_categories(&self, from_year: i32, to_year: i32) -> Result<(), Box<dyn Error>> {
        let to_year_path = self.db_path_from_year(to_year, Self::CATEGORIES_FILENAME);
        let from_year_path = self.db_path_from_year(from_year, Self::CATEGORIES_FILENAME);

        self.copy_file(&from_year_path, &to_year_path)?;

        Ok(())
    }

    pub fn copy_plans(&self, from_year: i32, to_year: i32) -> Result<(), Box<dyn Error>> {
        let to_year_path = self.db_path_from_year(to_year, Self::PLANS_FILENAME);
        let from_year_path = self.db_path_from_year(from_year, Self::PLANS_FILENAME);

        self.copy_file(&from_year_path, &to_year_path)?;

        Ok(())
    }

    fn copy_file(&self, from: &PathBuf, to: &PathBuf) -> Result<(), Box<dyn Error>> {
        let from_file = FileStorage::from(from).and_read()?;
        let mut to_file = FileStorage::from(to).and_create()?;

        to_file.content_set(from_file.content());
        to_file.write()?;

        Ok(())
    }

    pub fn available_reports(&self) -> Vec<i32> {
        self.available_for(Self::EXPENSES_FILENAME)
    }

    pub fn available_categories(&self) -> Vec<i32> {
        self.available_for(Self::CATEGORIES_FILENAME)
    }

    pub fn available_plans(&self) -> Vec<i32> {
        self.available_for(Self::PLANS_FILENAME)
    }

    fn available_for(&self, filename: &str) -> Vec<i32> {
        // 1. Read all valid subdirectories
        let root_dirs = fs::read_dir(&self.root_path)
            .expect("The root path supposed to be already validated!")
            .filter_map(|e| e.ok());

        // 2. Filter all directories that can be years
        let years = root_dirs.filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<i32>().ok())
        });

        // 3. Check if those dirs has wanted database
        let mut years_has_db: Vec<i32> = years
            .filter(|&year| {
                let path = self.db_path_from_year(year, filename);
                path.is_file()
            })
            .collect();

        years_has_db.sort_by(|a, b| b.cmp(a));
        years_has_db
    }

    fn create_storage_in_year(&self, year: i32, filename: &str) -> Result<(), StorageError> {
        let year = year.to_string();
        let file_path = self.root_path.join(&year).join(&filename);

        FileStorage::from(&file_path).create()?;

        Ok(())
    }

    fn db_path_from_year(&self, year: i32, filename: &str) -> PathBuf {
        let year = year.to_string();
        self.root_path.join(&year).join(&filename)
    }

    fn db_path_from_root(&self, filename: &str) -> PathBuf {
        self.root_path.join(&filename)
    }
}
