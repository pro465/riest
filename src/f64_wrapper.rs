use crate::program::Cost;
use std::cmp::*;
use std::f64::consts::*;
use std::fmt;

pub type Delta = f64;

#[derive(Clone, Copy, Debug, Default)]
pub struct F64Wrapper<'a> {
    value: f64,
    thres: f64,
    cost: Cost,
    name: Option<&'a str>,
}

impl<'a> F64Wrapper<'a> {
    pub const fn new(value: f64, thres: f64, cost: Cost, name: Option<&'a str>) -> Self {
        Self {
            value,
            thres,
            cost,
            name,
        }
    }
    pub const fn value(&self) -> f64 {
        self.value
    }
    pub fn value_mut(&mut self) -> &mut f64 {
        &mut self.value
    }
    pub const fn thres(&self) -> f64 {
        self.thres
    }
    pub const fn cost(&self) -> Cost {
        self.cost
    }
    pub const fn name(&self) -> Option<&'a str> {
        self.name
    }
    pub fn eqf(&self, other: f64) -> bool {
        (self.value - other).abs() < self.thres
    }
}

impl<'a> PartialEq for F64Wrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.eqf(other.value)
    }
}

impl<'a> Eq for F64Wrapper<'a> {}

impl<'a> PartialOrd for F64Wrapper<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self == other {
            Ordering::Equal
        } else {
            self.value.total_cmp(&other.value)
        })
    }
}

impl<'a> Ord for F64Wrapper<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a> fmt::Display for F64Wrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name {
            Some(n) => write!(f, "{}", n),
            None => write!(f, "{}", self.value),
        }
    }
}
