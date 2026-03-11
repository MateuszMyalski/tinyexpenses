use crate::balances::{Balances, BalancesMap};
use crate::models::csv::tables::Categories;
use crate::models::csv::types::{PlansEntry, ReportEntry};
use chrono::Datelike;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use tracing::log::warn;

#[derive(Debug)]
pub enum BudgetError {
    Report(Box<dyn Error>),
    Plans(Box<dyn Error>),
    Categories(Box<dyn Error>),
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetError::Report(e) => write!(f, "Report error: {}", e),
            BudgetError::Categories(e) => write!(f, "Categories error: {}", e),
            BudgetError::Plans(e) => write!(f, "Plans error: {}", e),
        }
    }
}

impl Error for BudgetError {}

pub struct Budget {
    balances: BalancesMap,
    initial_balance: f32,
    incomes_subcategories: Vec<String>,
}

impl Budget {
    pub fn new(categories: &Categories) -> Self {
        Budget {
            balances: BalancesMap::new(&categories.subcategories()),
            initial_balance: f32::default(),
            incomes_subcategories: categories
                .map_by_category()
                .get(Categories::INCOME_LABEL)
                .unwrap_or(&Vec::new())
                .to_vec(),
        }
    }

    pub fn load_from_plans(&mut self, content: &[PlansEntry]) {
        self.balances.reset();
        let mut rows_to_keep = HashSet::new();

        for record in content.iter() {
            for (idx, value) in record.values.iter().enumerate() {
                if let Err(_) = self
                    .balances
                    .set(record.subcategory.get(), value.get(), idx)
                {
                    // TODO(mmyalski) add to "filtered" group
                    break;
                }
            }

            rows_to_keep.insert(record.subcategory.get());
        }

        self.balances
            .retain(|label, _| rows_to_keep.contains(label.as_str()));

        self.initial_balance = 0.0;
    }

    pub fn load_from_report(&mut self, content: &[ReportEntry]) {
        self.balances.reset();

        for record in content.iter() {
            let Some(date) = record.date.get() else {
                warn!("Unable to read month from record '{}'", record);
                continue;
            };

            let subcategory = record.subcategory.get();

            let value = match self
                .incomes_subcategories
                .contains(&subcategory.to_string())
            {
                true => record.value.get(),
                false => record.value.get() * -1.0, // All non income subcategories are expenses
            };

            if let Err(_) = self
                .balances
                .add(subcategory, value, date.month0() as usize)
            {
                // TODO(mmyalski) add to "filtered" group
            }
        }

        if let Some(initial_balance_record) = content.iter().find(|record| {
            record
                .subcategory
                .get()
                .eq(Categories::INITIAL_BALANCE_LABEL)
        }) {
            self.initial_balance = initial_balance_record.value.get()
        } else {
            self.initial_balance = 0.0;
        }
    }

    pub fn balance(&self, net: &Balances) -> f32 {
        self.initial_balance + net.iter().sum::<f32>()
    }

    pub fn net(&self) -> Balances {
        self.balances.sum_by_col()
    }

    pub fn sum_of(&self, subcategories: &[String]) -> Balances {
        let mut sums = Balances::default();

        let filtered = self
            .balances
            .iter()
            .filter(|(label, _)| subcategories.contains(&label));

        for (_, record) in filtered {
            sums += record;
        }

        sums
    }

    pub fn balances(&self) -> &BalancesMap {
        &self.balances
    }
}
