use crate::models::csv::{Date, Field, Text, Timestamp, Value};
use chrono::{NaiveDate, Utc};

pub type Subcategory = Field<Text>;
pub type Category = Field<Text>;
pub type Description = Field<Text>;
pub type AccountName = Field<Text>;

pub type Amount = Field<Value>;
pub type EntryDate = Field<Date>;
pub type EntryTimestamp = Field<Timestamp>;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct SavingsEntry {
    pub subcategory: Subcategory,
    pub account_name: AccountName,
    pub value: Amount,
}

impl PartialEq<str> for SavingsEntry {
    fn eq(&self, other: &str) -> bool {
        self.subcategory.get().eq(other)
    }
}

impl SavingsEntry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_subcategory(mut self, s: &str) -> Self {
        self.subcategory.set(s);
        self
    }

    pub fn with_account_name(mut self, a: &str) -> Self {
        self.account_name.set(a);
        self
    }

    pub fn with_value(mut self, v: f32) -> Self {
        self.value.set(v);
        self
    }
}

#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReportEntry {
    pub timestamp: EntryTimestamp,
    pub subcategory: Subcategory,
    pub date: EntryDate,
    pub value: Amount,
    pub description: Description,
}

impl std::fmt::Display for ReportEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {}, {})",
            self.timestamp.raw(),
            self.subcategory.raw(),
            self.date.raw(),
            self.value.raw(),
            self.description.raw()
        )
    }
}

impl ReportEntry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_curr_timestamp(self) -> Self {
        self.with_timestamp(Utc::now().timestamp())
    }

    pub fn with_timestamp(mut self, secs: i64) -> Self {
        self.timestamp.set(secs);
        self
    }

    pub fn with_value(mut self, v: f32) -> Self {
        self.value.set(v);
        self
    }

    pub fn with_curr_date(self) -> Self {
        self.with_date(&Utc::now().date_naive())
    }

    pub fn with_date(mut self, date: &NaiveDate) -> Self {
        self.date.set(date);
        self
    }

    pub fn with_subcategory(mut self, s: &str) -> Self {
        self.subcategory.set(s);
        self
    }

    pub fn with_description(mut self, d: &str) -> Self {
        self.description.set(d);
        self
    }
}

#[derive(Clone, Default, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct CategoriesEntry {
    pub subcategory: Subcategory,
    pub category: Category,
}

impl CategoriesEntry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_subcategory(mut self, s: &str) -> Self {
        self.subcategory.set(s);
        self
    }

    pub fn with_category(mut self, c: &str) -> Self {
        self.category.set(c);
        self
    }
}

impl PartialEq<str> for CategoriesEntry {
    fn eq(&self, other: &str) -> bool {
        self.subcategory.get().eq(other)
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct PlansEntry {
    pub subcategory: Subcategory,
    pub values: [Amount; 12],
}
