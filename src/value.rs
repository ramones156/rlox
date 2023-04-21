use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;
use std::string::ParseError;

use anyhow::{anyhow, Result};

use crate::object::Object;

impl From<Value> for Option<u8> {
    fn from(val: Value) -> Self {
        match val {
            Value::VAL_BOOL(b) => Some(b as u8),
            Value::VAL_NIL => None,
            Value::VAL_NUMBER(n) => Some(n as u8),
            Value::VAL_OBJECT(o) => Some(o.into()),
        }
    }
}

impl Add for Value {
    type Output = Result<f32>;

    fn add(self, rhs: Self) -> Self::Output {
        if let Value::VAL_NUMBER(lhs) = self {
            if let Value::VAL_NUMBER(rhs) = rhs {
                return Ok(lhs + rhs);
            }
        }
        Err(anyhow!("Value must be a number"))
    }
}

impl Mul for Value {
    type Output = Result<f32>;

    fn mul(self, rhs: Self) -> Self::Output {
        if let Value::VAL_NUMBER(lhs) = self {
            if let Value::VAL_NUMBER(rhs) = rhs {
                return Ok(lhs * rhs);
            }
        }
        Err(anyhow!("Value must be a number"))
    }
}

impl Div for Value {
    type Output = Result<f32>;

    fn div(self, rhs: Self) -> Self::Output {
        if let Value::VAL_NUMBER(lhs) = self {
            if let Value::VAL_NUMBER(rhs) = rhs {
                return Ok(lhs / rhs);
            }
        }
        Err(anyhow!("Value must be a number"))
    }
}

impl Sub for Value {
    type Output = Result<f32>;

    fn sub(self, rhs: Self) -> Self::Output {
        if let Value::VAL_NUMBER(lhs) = self {
            if let Value::VAL_NUMBER(rhs) = rhs {
                return Ok(lhs - rhs);
            }
        }
        Err(anyhow!("Value must be a number"))
    }
}

impl Neg for Value {
    type Output = Result<Value>;

    fn neg(self) -> Self::Output {
        if let Value::VAL_NUMBER(lhs) = self {
            return Ok(Self::VAL_NUMBER(-lhs));
        }
        Err(anyhow!("Value must be a number"))
    }
}

impl FromStr for Value {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = s.parse::<f32>();
        match result {
            Ok(result) => Ok(Self::VAL_NUMBER(result)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    VAL_BOOL(bool),
    VAL_NIL,
    VAL_NUMBER(f32),
    VAL_OBJECT(Object),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::VAL_BOOL(b) => writeln!(f, "{b}"),
            Value::VAL_NIL => writeln!(f, "nil"),
            Value::VAL_NUMBER(n) => writeln!(f, "{n}"),
            Value::VAL_OBJECT(o) => writeln!(f, "{o}"),
        }
    }
}

#[derive(Default)]
pub struct ValueArray {
    pub count: usize,
    pub(crate) values: Vec<Value>,
}

impl ValueArray {
    pub fn write(&mut self, value: Value) {
        if self.values.len() < self.count + 1 {
            self.values.push(value);
        } else {
            self.values[self.count] = value;
        }
        self.count += 1;
    }
}
