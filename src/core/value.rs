use crate::core::units::CompoundUnit;
use std::fmt;

/// Represents a computed value in the calculator.
/// Supports f64 numbers with optional compound unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub number: f64,
    pub unit: Option<CompoundUnit>,
    pub display_str: Option<String>,
}

impl Value {
    pub fn new(number: f64) -> Self {
        Self {
            number,
            unit: None,
            display_str: None,
        }
    }

    pub fn with_unit(number: f64, unit: CompoundUnit) -> Self {
        Self {
            number,
            unit: Some(unit),
            display_str: None,
        }
    }

    pub fn is_nan(&self) -> bool {
        self.number.is_nan()
    }

    #[allow(dead_code)]
    pub fn is_infinite(&self) -> bool {
        self.number.is_infinite()
    }

    #[allow(dead_code)]
    pub fn has_unit(&self) -> bool {
        self.unit.is_some()
    }

    pub fn get_unit_string(&self) -> Option<String> {
        self.unit.as_ref().map(|u| u.to_string())
    }

    pub fn number(&self) -> f64 {
        self.number
    }

    pub fn with_display_str(mut self, display: String) -> Self {
        self.display_str = Some(display);
        self
    }

    /// Check if this value has compatible dimensions with another
    pub fn dimensions_compatible(&self, other: &Self) -> bool {
        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => {
                if let (Ok((d1, _)), Ok((d2, _))) =
                    (u1.to_dimensions_and_factor(), u2.to_dimensions_and_factor())
                {
                    d1.is_compatible(&d2)
                } else {
                    false
                }
            }
            (None, None) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.number.is_nan() {
            write!(f, "NaN")
        } else if self.number.is_infinite() {
            if self.number > 0.0 {
                write!(f, "∞")
            } else {
                write!(f, "-∞")
            }
        } else if let Some(ref unit) = self.unit {
            write!(f, "{} {}", self.number, unit)
        } else {
            write!(f, "{}", self.number)
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::new(n)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Self::new(n as f64)
    }
}
