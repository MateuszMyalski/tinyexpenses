use crate::budget::Budget;
use crate::models::csv::tables::Categories;
use crate::models::csv::types::PlansEntry;
use crate::models::resources;

pub const MONTHS: [&'static str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn format_number(n: f32) -> String {
    let is_neg = n < 0.0;
    let number = format!("{:.2}", f32::abs(n));
    let mut split = number.split('.');

    let Some(integer) = split.next() else {
        return number;
    };

    let Some(frac) = split.next() else {
        return number;
    };

    let mut integer_separated = String::new();
    let mut cnt = 0;

    for c in integer.chars().rev() {
        if cnt > 0 && cnt % 3 == 0 {
            integer_separated.push(' ');
        }

        integer_separated.push(c);
        cnt += 1;
    }

    let integer_separated: String = integer_separated.chars().rev().collect();

    format!(
        "{}{}.{}",
        if is_neg { "-" } else { "" },
        integer_separated,
        frac
    )
}

pub trait DatabasePickerView {
    fn options(&self) -> Vec<(String, String)>;
    fn selected(&self) -> String;
}

/*
 * |------------|---|---------| ... |---------|-------|
 * | Subcat.    | C | mnth 1  | ... | mnth 12 | Total |
 *                          ...                       |
 * | Net()      | . |           ...                   | (sum per column)
 * | SumOf(...) | . |           ...                   | (sum per subcat.) sum_income
 * | SumOf(...) | . |           ...                   | (sum per subcat.) sum_savings
 * | SumOf(...) | . |           ...                   | (sum per subcat.) sum_needs
 * | SumOf(...) | . |           ...                   | (sum per subcat.) sum_wants
 *
 */
pub trait BudgetSummaryView {
    type Cells;

    fn headers(&self) -> Self::Cells;
    fn subcategories(&self) -> Vec<Self::Cells>;
    fn net(&self) -> Self::Cells;
    fn sum_income(&self) -> Self::Cells;
    fn sum_savings(&self) -> Self::Cells;
    fn sum_savings_prcnt(&self) -> Self::Cells;
    fn sum_wants(&self) -> Self::Cells;
    fn sum_wants_prcnt(&self) -> Self::Cells;
    fn sum_needs(&self) -> Self::Cells;
    fn sum_needs_prcnt(&self) -> Self::Cells;
    fn balance(&self) -> String;
}

pub trait ValuesAttributes {
    fn attributes(&self, subcategory: &str, month0: usize) -> String;
}

pub struct YearReportSummaryView {
    res: resources::BudgetSummary,
}

/*
* |----------------------------------|
* | Account name | total             |
* |              |                   |
* | El 1         | balance | prcnt   |
* | El 2         | balance | prcnt   |
*               ...
* | El n         | balance | prcnt   |

*/
pub trait SavingsAccountView {
    type ElementsInfoCells;

    fn name(&self) -> String;
    fn elements(&self) -> Vec<Self::ElementsInfoCells>;
    fn total(&self) -> String;
}

impl YearReportSummaryView {
    pub fn new(budget: &Budget, categories: &Categories) -> Self {
        YearReportSummaryView {
            res: resources::BudgetSummary::new(budget, categories),
        }
    }

    fn make_row(
        &self,
        label: &str,
        sub_label: &str,
        values: &[f32],
        total: f32,
    ) -> <YearReportSummaryView as BudgetSummaryView>::Cells {
        let mut row = <YearReportSummaryView as BudgetSummaryView>::Cells::default();

        row[0] = label.to_string();
        row[1] = sub_label.to_string();

        for (idx, value) in values.iter().enumerate() {
            row[2 + idx] = format_number(*value);
        }

        row[2 + values.len()] = format_number(total);

        row
    }

    pub fn compare_with_plans(
        &mut self,
        plan_entries: &[PlansEntry],
        income_categories: &[String],
    ) {
        self.res.compare_with_plans(plan_entries, income_categories);
    }
}

impl ValuesAttributes for YearReportSummaryView {
    fn attributes(&self, subcategory: &str, month0: usize) -> String {
        self.res
            .values_attr
            .get(subcategory)
            .map_or("".to_string(), |attrs| attrs[month0].to_string())
    }
}

impl BudgetSummaryView for YearReportSummaryView {
    type Cells = [String; 15];

    fn headers(&self) -> Self::Cells {
        [
            "Subcategory".to_string(),
            "Type".to_string(),
            MONTHS[0][0..3].to_string(),
            MONTHS[1][0..3].to_string(),
            MONTHS[2][0..3].to_string(),
            MONTHS[3][0..3].to_string(),
            MONTHS[4][0..3].to_string(),
            MONTHS[5][0..3].to_string(),
            MONTHS[6][0..3].to_string(),
            MONTHS[7][0..3].to_string(),
            MONTHS[8][0..3].to_string(),
            MONTHS[9][0..3].to_string(),
            MONTHS[10][0..3].to_string(),
            MONTHS[11][0..3].to_string(),
            "Total".to_string(),
        ]
    }

    fn subcategories(&self) -> Vec<Self::Cells> {
        let mut rows = Vec::<Self::Cells>::new();
        let mut vectorized = Vec::from_iter(self.res.subcategories.iter());
        vectorized.sort_by_key(|(_, (category, _))| category.to_string());

        for (subcategory, (category, balance)) in &vectorized {
            let row = self.make_row(subcategory, &category, &balance.as_ref(), balance.total());
            rows.push(row);
        }

        rows
    }

    fn balance(&self) -> String {
        format_number(self.res.balance)
    }

    fn net(&self) -> Self::Cells {
        self.make_row("Net", "", self.res.net.as_ref(), self.res.net.total())
    }

    fn sum_income(&self) -> Self::Cells {
        self.make_row(
            Categories::INCOME_LABEL,
            "",
            self.res.sum_income.as_ref(),
            self.res.sum_income.total(),
        )
    }

    fn sum_needs(&self) -> Self::Cells {
        self.make_row(
            Categories::NEEDS_LABEL,
            "",
            self.res.sum_needs.as_ref(),
            self.res.sum_needs.total(),
        )
    }

    fn sum_savings(&self) -> Self::Cells {
        self.make_row(
            Categories::SAVINGS_LABEL,
            "",
            self.res.sum_savings.as_ref(),
            self.res.sum_savings.total(),
        )
    }

    fn sum_wants(&self) -> Self::Cells {
        self.make_row(
            Categories::WANTS_LABEL,
            "",
            self.res.sum_wants.as_ref(),
            self.res.sum_wants.total(),
        )
    }

    fn sum_needs_prcnt(&self) -> Self::Cells {
        self.make_row(
            Categories::NEEDS_LABEL,
            "",
            self.res.sum_needs_prcnt.as_ref(),
            self.res.sum_needs_prcnt.total_prcnt(),
        )
    }

    fn sum_savings_prcnt(&self) -> Self::Cells {
        self.make_row(
            Categories::SAVINGS_LABEL,
            "",
            self.res.sum_savings_prcnt.as_ref(),
            self.res.sum_savings_prcnt.total_prcnt(),
        )
    }

    fn sum_wants_prcnt(&self) -> Self::Cells {
        self.make_row(
            Categories::WANTS_LABEL,
            "",
            self.res.sum_wants_prcnt.as_ref(),
            self.res.sum_wants_prcnt.total_prcnt(),
        )
    }
}

pub struct MonthReportSummaryView {
    res: resources::BudgetSummary,
    month0: usize,
}

impl MonthReportSummaryView {
    pub fn new(budget: &Budget, categories: &Categories, month0: usize) -> Self {
        assert!(month0 < MONTHS.len());

        MonthReportSummaryView {
            res: resources::BudgetSummary::new(budget, categories),
            month0: month0,
        }
    }

    pub fn compare_with_plans(
        &mut self,
        plan_entries: &[PlansEntry],
        income_categories: &[String],
    ) {
        self.res.compare_with_plans(plan_entries, income_categories);
    }
}

impl ValuesAttributes for MonthReportSummaryView {
    fn attributes(&self, subcategory: &str, month0: usize) -> String {
        self.res
            .values_attr
            .get(subcategory)
            .map_or("".to_string(), |attrs| attrs[month0].to_string())
    }
}

impl BudgetSummaryView for MonthReportSummaryView {
    type Cells = [String; 3];

    fn headers(&self) -> Self::Cells {
        [
            "Subcategory".to_string(),
            "Type".to_string(),
            MONTHS[self.month0].to_string(),
        ]
    }

    fn balance(&self) -> String {
        format_number(self.res.balance)
    }

    fn net(&self) -> Self::Cells {
        [
            "Net".to_string(),
            "".to_string(),
            format_number(self.res.net[self.month0]),
        ]
    }

    fn subcategories(&self) -> Vec<Self::Cells> {
        let mut rows = Vec::<Self::Cells>::new();
        let mut vectorized = Vec::from_iter(self.res.subcategories.iter());
        vectorized.sort_by_key(|(_, (category, _))| category.to_string());

        for (subcategory, (category, balance)) in &vectorized {
            let row = [
                subcategory.to_string(),
                category.clone(),
                format_number(balance[self.month0]).to_string(),
            ];

            rows.push(row);
        }

        rows
    }

    fn sum_income(&self) -> Self::Cells {
        [
            Categories::INCOME_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_income[self.month0]),
        ]
    }

    fn sum_needs(&self) -> Self::Cells {
        [
            Categories::NEEDS_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_needs[self.month0]),
        ]
    }

    fn sum_savings(&self) -> Self::Cells {
        [
            Categories::SAVINGS_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_savings[self.month0]),
        ]
    }

    fn sum_wants(&self) -> Self::Cells {
        [
            Categories::WANTS_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_wants[self.month0]),
        ]
    }

    fn sum_wants_prcnt(&self) -> Self::Cells {
        [
            Categories::WANTS_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_wants_prcnt[self.month0]),
        ]
    }

    fn sum_needs_prcnt(&self) -> Self::Cells {
        [
            Categories::NEEDS_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_needs_prcnt[self.month0]),
        ]
    }

    fn sum_savings_prcnt(&self) -> Self::Cells {
        [
            Categories::SAVINGS_LABEL.to_string(),
            "".to_string(),
            format_number(self.res.sum_savings_prcnt[self.month0]),
        ]
    }
}

pub struct PlansSummaryView(YearReportSummaryView);

impl PlansSummaryView {
    pub fn new(budget: &Budget, categories: &Categories) -> Self {
        PlansSummaryView(YearReportSummaryView::new(budget, categories))
    }
}

impl std::ops::Deref for PlansSummaryView {
    type Target = YearReportSummaryView;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ByYearDatabasePickerView {
    res: resources::DatabasePicker,
    endpoint: String,
}

impl ByYearDatabasePickerView {
    pub fn new(avails: &Vec<i32>, active_year: i32, endpoint: &str) -> Self {
        ByYearDatabasePickerView {
            res: resources::DatabasePicker {
                avails: avails.clone(),
                active: active_year,
            },
            endpoint: endpoint.to_string(),
        }
    }
}

impl DatabasePickerView for ByYearDatabasePickerView {
    fn options(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();

        for year in &self.res.avails {
            let value = format!("{}{}", self.endpoint, year);
            attrs.push((value, year.to_string()));
        }

        attrs
    }

    fn selected(&self) -> String {
        self.res.active.to_string()
    }
}

pub struct ByMonthDatabasePickerView {
    res: resources::DatabasePicker,
    year: i32,
    endpoint: String,
}

impl ByMonthDatabasePickerView {
    pub fn selected_as_value(&self) -> String {
        self.res.active.to_string()
    }
}

impl ByMonthDatabasePickerView {
    pub fn new(active_year: i32, active_month0: i32, endpoint: &str) -> Self {
        assert!(active_month0 >= 0 && active_month0 <= 12);

        ByMonthDatabasePickerView {
            res: resources::DatabasePicker {
                avails: (1..=12).collect(),
                active: active_month0,
            },
            year: active_year,
            endpoint: endpoint.to_string(),
        }
    }
}

impl DatabasePickerView for ByMonthDatabasePickerView {
    fn options(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();

        for month in &self.res.avails {
            let value = format!("{}{}/{}", self.endpoint, self.year, month);
            let month0 = (month - 1) as usize;

            attrs.push((value, MONTHS[month0].to_string()));
        }

        attrs
    }

    fn selected(&self) -> String {
        let month0 = (self.res.active - 1) as usize;

        MONTHS[month0].to_string()
    }
}

impl SavingsAccountView for resources::SavingsAccount {
    type ElementsInfoCells = (String, String, String);
    fn elements(&self) -> Vec<Self::ElementsInfoCells> {
        let mut elements = Vec::new();

        for (el_label, el_balance) in &self.balances {
            let el_balance_prcnt = self
                .balances_prcnt
                .get(el_label)
                .copied()
                .unwrap_or_default();

            elements.push((
                el_label.to_string(),
                format_number(*el_balance),
                format_number(el_balance_prcnt),
            ))
        }

        elements
    }

    fn name(&self) -> String {
        self.account_name.to_string()
    }

    fn total(&self) -> String {
        format_number(self.total)
    }
}
