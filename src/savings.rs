use crate::models::csv::types::SavingsEntry;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct SavingRecord {
    pub subcategory: String,
    pub balance: f32,
}

pub struct SavingsCollection(BTreeMap<String, Vec<SavingRecord>>);

impl std::ops::Deref for SavingsCollection {
    type Target = BTreeMap<String, Vec<SavingRecord>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SavingsCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SavingsCollection {
    pub fn new(savings: &[SavingsEntry]) -> Self {
        let mut collection = BTreeMap::new();

        for e in savings.iter() {
            let account = collection
                .entry(e.account_name.get().to_string())
                .or_insert(Vec::new());

            account.push(SavingRecord {
                subcategory: e.subcategory.get().to_string(),
                balance: e.value.get(),
            });
        }

        SavingsCollection(collection)
    }
}
