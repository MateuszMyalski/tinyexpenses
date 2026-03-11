use std::collections::HashMap;
use std::ops;

#[derive(Default, Debug, Clone)]
pub struct Balances([f32; 12]);

impl Balances {
    pub fn total(&self) -> f32 {
        self.iter().sum()
    }

    pub fn total_prcnt(&self) -> f32 {
        let sum: f32 = self.iter().sum();
        sum / self.len() as f32
    }

    pub fn abs(val: &Balances) -> Balances {
        let mut out = Balances::default();

        for idx in 0..val.len() {
            out[idx] = f32::abs(val[idx]);
        }
        out
    }

    fn div_impl(lhs: &Balances, rhs: &Balances) -> Balances {
        let mut out = Balances::default();

        for i in 0..lhs.len() {
            let denom = rhs[i];
            out[i] = if denom != 0.0 { lhs[i] / denom } else { 0.0 };
        }
        out
    }

    fn mul_impl(lhs: &Balances, rhs: &Balances) -> Balances {
        let mut out = Balances::default();

        for i in 0..lhs.len() {
            out[i] = lhs[i] * rhs[i];
        }
        out
    }
}

impl ops::AddAssign for Balances {
    fn add_assign(&mut self, rhs: Self) {
        for idx in 0..self.len() {
            self[idx] += rhs[idx];
        }
    }
}

impl ops::AddAssign<&Self> for Balances {
    fn add_assign(&mut self, rhs: &Self) {
        for idx in 0..self.len() {
            self[idx] += rhs[idx];
        }
    }
}

impl ops::Div<&Self> for &Balances {
    type Output = Balances;

    fn div(self, rhs: &Self) -> Self::Output {
        Balances::div_impl(&self, &rhs)
    }
}

impl ops::Div<Self> for Balances {
    type Output = Balances;

    fn div(self, rhs: Self) -> Self::Output {
        Balances::div_impl(&self, &rhs)
    }
}

impl ops::Div<&Self> for Balances {
    type Output = Balances;

    fn div(self, rhs: &Self) -> Self::Output {
        Balances::div_impl(&self, rhs)
    }
}

impl ops::Mul<&Self> for Balances {
    type Output = Balances;

    fn mul(self, rhs: &Self) -> Self::Output {
        Balances::mul_impl(&self, rhs)
    }
}

impl ops::Mul<Self> for Balances {
    type Output = Balances;

    fn mul(self, rhs: Self) -> Self::Output {
        Balances::mul_impl(&self, &rhs)
    }
}

impl ops::Mul<Self> for &Balances {
    type Output = Balances;

    fn mul(self, rhs: Self) -> Self::Output {
        Balances::mul_impl(self, rhs)
    }
}

impl ops::Mul<&Self> for &Balances {
    type Output = Balances;

    fn mul(self, rhs: &Self) -> Self::Output {
        Balances::mul_impl(self, rhs)
    }
}

impl ops::Mul<f32> for &Balances {
    type Output = Balances;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut out = Balances::default();

        for i in 0..self.len() {
            out[i] = self[i] * rhs;
        }
        out
    }
}

impl ops::Deref for Balances {
    type Target = [f32; 12];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Balances {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct BalancesMap(HashMap<String, Balances>);

impl std::ops::Deref for BalancesMap {
    type Target = HashMap<String, Balances>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BalancesMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BalancesMap {
    pub fn new(labels: &Vec<String>) -> Self {
        BalancesMap(HashMap::from_iter(
            labels
                .iter()
                .map(|label| (label.clone(), Balances::default())),
        ))
    }

    pub fn sum_by_col(&self) -> Balances {
        let mut sums = Balances::default();

        for row in self.values() {
            sums += row;
        }

        sums
    }

    pub fn add(&mut self, subcategory: &str, diff: f32, month0: usize) -> Result<(), ()> {
        let Some(row) = self.get_mut(subcategory) else {
            return Err(());
        };

        let Some(value) = row.get_mut(month0) else {
            return Err(());
        };

        *value += diff;

        Ok(())
    }

    pub fn set(&mut self, subcategory: &str, new_value: f32, month0: usize) -> Result<(), ()> {
        let Some(row) = self.get_mut(subcategory) else {
            return Err(());
        };

        let Some(value) = row.get_mut(month0) else {
            return Err(());
        };

        *value = new_value;

        Ok(())
    }

    pub fn reset(&mut self) {
        for row in self.values_mut() {
            *row = Balances::default();
        }
    }
}
