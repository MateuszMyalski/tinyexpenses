use crate::models::csv::types::{CategoriesEntry, PlansEntry, ReportEntry, SavingsEntry};
use crate::storage::CsvFile;
use std::collections::BTreeMap;

pub type Categories = CsvFile<CategoriesEntry>;

impl Categories {
    pub const INCOME_LABEL: &str = "Income";
    pub const SAVINGS_LABEL: &str = "Savings";
    pub const NEEDS_LABEL: &str = "Needs";
    pub const WANTS_LABEL: &str = "Wants";
    pub const INITIAL_BALANCE_LABEL: &str = "Initial Balance";

    pub fn contains(&self, record: &CategoriesEntry) -> bool {
        self.content().contains(record)
    }

    pub fn is_savings(&self, subcategory: &str) -> bool {
        self.content()
            .iter()
            .find(|e| e.eq(&subcategory) && e.category.get().eq(Self::SAVINGS_LABEL))
            .is_some()
    }

    pub fn map_by_category(&self) -> BTreeMap<String, Vec<String>> {
        // We use HashMap with sort feature to always display data in the same order
        // despite of different records order in database.
        let mut map = BTreeMap::new();

        for entry in self.content().iter() {
            map.entry(entry.category.get().to_string())
                .or_insert(Vec::new())
                .push(entry.subcategory.get().to_string());
        }

        map
    }

    pub fn map_by_subcategory(&self) -> BTreeMap<String, String> {
        // We use HashMap with sort feature to always display data in the same order
        // despite of different records order in database.
        let mut map = BTreeMap::new();

        for entry in self.content().iter() {
            map.insert(
                entry.subcategory.get().to_string(),
                entry.category.get().to_string(),
            );
        }

        map
    }

    pub fn subcategories(&self) -> Vec<String> {
        Vec::from_iter(
            self.content()
                .iter()
                .map(|record| record.subcategory.get().to_string()),
        )
    }
}

pub type Report = CsvFile<ReportEntry>;

pub type Plans = CsvFile<PlansEntry>;

pub type Savings = CsvFile<SavingsEntry>;

impl Savings {
    pub fn get(&self, subcategory: &str) -> Option<&SavingsEntry> {
        self.content().iter().find(|e| e.eq(&subcategory))
    }

    pub fn extract(&mut self, subcategory: &str) -> Option<SavingsEntry> {
        let findings: Vec<SavingsEntry> = self
            .content_mut()
            .extract_if(.., |e| e.eq(&subcategory))
            .collect();

        if findings.is_empty() {
            return None;
        }

        // In case we found multiple records, join them and return as one.
        // This should fix all duplicated entries in the file automatically
        Some(
            SavingsEntry::new()
                .with_account_name(findings[0].account_name.get())
                .with_subcategory(subcategory)
                .with_value(findings.iter().map(|f| f.value.get()).sum()),
        )
    }

    pub fn update(&mut self, subcategory: &str) -> &mut SavingsEntry {
        if let Some(pos) = self.content_mut().iter().position(|e| e.eq(subcategory)) {
            return &mut self.content_mut()[pos];
        }

        let entry = SavingsEntry::new()
            .with_subcategory(subcategory)
            .with_account_name(subcategory);

        self.content_mut().push(entry);
        self.content_mut().last_mut().unwrap()
    }
}
