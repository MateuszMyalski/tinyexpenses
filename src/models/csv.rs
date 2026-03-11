use chrono::{DateTime, NaiveDate, NaiveDateTime};
use std::{borrow::Cow, marker::PhantomData};

pub mod tables;
pub mod types;

fn sanitize_string(input: &str) -> Cow<'_, str> {
    if input.contains(['\r', '\n']) || input.trim().len() != input.len() {
        Cow::Owned(
            input
                .replace("\r\n", " ")
                .replace("\r", " ")
                .replace("\n", " ")
                .trim()
                .to_string(),
        )
    } else {
        Cow::Borrowed(input)
    }
}

#[derive(Clone, serde::Serialize)]
pub struct Field<T> {
    raw: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

impl<T> Default for Field<T> {
    fn default() -> Self {
        Self {
            raw: String::new(),
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Field<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.eq(other.raw())
    }
}

impl<'de, T> serde::Deserialize<'de> for Field<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Field {
            raw: String::deserialize(deserializer).unwrap_or_default(),
            _marker: PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct Text;

#[derive(Clone)]
pub struct Value;

#[derive(Clone)]
pub struct Date;

#[derive(Clone)]
pub struct Timestamp;

impl Field<Text> {
    pub fn set(&mut self, input: &str) {
        self.raw = sanitize_string(input).to_string();
    }

    pub fn get(&self) -> &str {
        &self.raw
    }
}

impl Field<Value> {
    fn round(x: f32) -> f32 {
        (x * 100.0).round() / 100.0
    }

    pub fn set(&mut self, v: f32) {
        self.raw = format!("{:.2}", Self::round(v));
    }

    pub fn get(&self) -> f32 {
        Self::round(self.raw.trim().parse::<f32>().unwrap_or_default())
    }

    pub fn change_by(&mut self, v: f32) {
        self.set(self.get() + v);
    }
}

impl Field<Date> {
    pub fn set(&mut self, date: &NaiveDate) {
        self.raw = date.format("%Y-%m-%d").to_string();
    }

    pub fn get(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.raw, "%Y-%m-%d").ok()
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl Field<Timestamp> {
    pub fn set(&mut self, secs: i64) {
        let ts = DateTime::from_timestamp(secs, 0).expect("Invalid timestamp");
        self.raw = ts.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    #[allow(dead_code)]
    pub fn get(&self) -> Option<NaiveDateTime> {
        NaiveDateTime::parse_from_str(&self.raw, "%Y-%m-%d %H:%M:%S").ok()
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}
