use crate::balances::Balances;
use crate::budget::Budget;
use crate::models::csv::tables::Categories;
use crate::models::csv::types::PlansEntry;
use crate::savings::SavingRecord;
use std::collections::{BTreeMap, HashMap};

pub struct DatabasePicker {
    pub avails: Vec<i32>,
    pub active: i32,
}

// pub type SavingsSummary = Vec<Account>;

pub struct BudgetSummary {
    pub subcategories: BTreeMap<String, (String, Balances)>,
    pub values_attr: HashMap<String, [String; 12]>,
    pub net: Balances,
    pub sum_income: Balances,
    pub sum_wants: Balances,
    pub sum_savings: Balances,
    pub sum_needs: Balances,
    pub sum_wants_prcnt: Balances,
    pub sum_savings_prcnt: Balances,
    pub sum_needs_prcnt: Balances,
    pub balance: f32,
}

impl BudgetSummary {
    pub fn new(budget: &Budget, categories: &Categories) -> Self {
        let balances = budget.balances();
        let subcategories_map = categories.map_by_subcategory();
        let categories_map = categories.map_by_category();

        let mut summary = BudgetSummary {
            subcategories: BTreeMap::new(),
            net: budget.net(),
            sum_wants_prcnt: Balances::default(),
            sum_savings_prcnt: Balances::default(),
            sum_needs_prcnt: Balances::default(),
            balance: f32::default(),
            sum_income: budget.sum_of(
                categories_map
                    .get(Categories::INCOME_LABEL)
                    .unwrap_or(&Vec::new())
                    .as_slice(),
            ),
            sum_savings: budget.sum_of(
                categories_map
                    .get(Categories::SAVINGS_LABEL)
                    .unwrap_or(&Vec::new())
                    .as_slice(),
            ),
            sum_needs: budget.sum_of(
                categories_map
                    .get(Categories::NEEDS_LABEL)
                    .unwrap_or(&Vec::new())
                    .as_slice(),
            ),
            sum_wants: budget.sum_of(
                categories_map
                    .get(Categories::WANTS_LABEL)
                    .unwrap_or(&Vec::new())
                    .as_slice(),
            ),
            values_attr: HashMap::new(),
        };

        for (label, balance) in balances.iter() {
            let Some(category) = subcategories_map.get(label) else {
                continue;
            };

            summary
                .subcategories
                .insert(label.to_string(), (category.to_string(), balance.clone()));
        }

        summary.balance = budget.balance(&summary.net);

        summary.sum_wants_prcnt =
            Balances::abs(&(&summary.sum_wants * 100.0)) / &summary.sum_income;
        summary.sum_savings_prcnt =
            Balances::abs(&(&summary.sum_savings * 100.0)) / &summary.sum_income;
        summary.sum_needs_prcnt =
            Balances::abs(&(&summary.sum_needs * 100.0)) / &summary.sum_income;

        summary
    }

    pub fn compare_with_plans(
        &mut self,
        plan_entries: &[PlansEntry],
        income_categories: &[String],
    ) {
        for plan_record in plan_entries {
            let Some((_, balances)) = self.subcategories.get(plan_record.subcategory.get()) else {
                continue;
            };

            let mut attributes: [String; 12] = Default::default();
            for month0 in 0..balances.len() {
                if income_categories.contains(&plan_record.subcategory.get().to_string()) {
                    attributes[month0] = if balances[month0] > plan_record.values[month0].get() {
                        "overperformance".to_string()
                    } else {
                        String::default()
                    };
                } else {
                    attributes[month0] = if balances[month0] < plan_record.values[month0].get() {
                        "overspending".to_string()
                    } else {
                        String::default()
                    };
                }
            }

            self.values_attr
                .insert(plan_record.subcategory.get().to_string(), attributes);
        }
    }
}

pub struct SavingsAccount {
    pub account_name: String,
    pub balances: BTreeMap<String, f32>,
    pub balances_prcnt: BTreeMap<String, f32>,
    pub total: f32,
}

impl SavingsAccount {
    pub fn new(account_name: &str, records: &[SavingRecord]) -> Self {
        let mut balances = BTreeMap::new();
        let mut balances_prcnt = BTreeMap::new();
        let mut total: f32 = 0.0;

        for record in records {
            balances.insert(record.subcategory.to_string(), record.balance);
            total += record.balance;
        }

        for (label, balance) in &balances {
            let balance_prcnt: f32 = if total > 0.0 {
                balance / total * 100.0
            } else {
                0.0
            };

            balances_prcnt.insert(label.to_string(), balance_prcnt);
        }

        SavingsAccount {
            account_name: account_name.to_string(),
            balances: balances,
            balances_prcnt: balances_prcnt,
            total: total,
        }
    }
}
