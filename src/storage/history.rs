use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Represents the workspace state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct WorkspaceState {
    pub variables: HashMap<String, StoredValue>,
    pub functions: HashMap<String, UserFunctionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StoredValue {
    Detailed { number: f64, unit: Option<String> },
    Legacy(f64),
}

impl StoredValue {
    pub fn from_value(value: &crate::core::value::Value) -> Self {
        Self::Detailed {
            number: value.number(),
            unit: value.unit.as_ref().map(ToString::to_string),
        }
    }

    pub fn to_value(&self) -> crate::core::value::Value {
        let (number, unit) = match self {
            Self::Detailed { number, unit } => (*number, unit.as_deref()),
            Self::Legacy(number) => (*number, None),
        };

        if let Some(unit) = unit {
            if let Ok(compound) = crate::core::units::parse_compound_unit(unit) {
                return crate::core::value::Value::with_unit(number, compound);
            }
        }

        crate::core::value::Value::new(number)
    }
}

/// Serializable function definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: String, // Store as string representation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
    pub is_error: bool,
    pub timestamp: u64,
    pub workspace: WorkspaceState, // Snapshot of variables and functions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
    pub max_entries: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000,
        }
    }

    pub fn add(
        &mut self,
        expression: String,
        result: String,
        is_error: bool,
        workspace: WorkspaceState,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.push(HistoryEntry {
            expression,
            result,
            is_error,
            timestamp,
            workspace,
        });

        if self.entries.len() > self.max_entries {
            let excess = self.entries.len() - self.max_entries;
            self.entries.drain(..excess);
        }
    }

    pub fn get_expressions(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.expression.as_str()).collect()
    }

    pub fn get_workspace_at(&self, index: usize) -> Option<&WorkspaceState> {
        self.entries.get(index).map(|e| &e.workspace)
    }

    pub fn last_n(&self, n: usize) -> Vec<&HistoryEntry> {
        let len = self.entries.len();
        if len <= n {
            self.entries.iter().collect()
        } else {
            self.entries[len - n..].iter().collect()
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::history_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::history_path()?;
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let history: History = serde_json::from_str(&content)?;
        Ok(history)
    }

    fn history_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("engcalc");
        Ok(dir.join("history.json"))
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::StoredValue;
    use crate::core::formatter;
    use crate::core::units;
    use crate::core::value::Value;

    #[test]
    fn stored_value_preserves_units() {
        let unit = units::parse_compound_unit("m/s").unwrap();
        let value = Value::with_unit(10.0, unit);

        let stored = StoredValue::from_value(&value);
        let restored = stored.to_value();

        assert_eq!(formatter::format_value(&restored), "10 m/s");
    }

    #[test]
    fn stored_value_reads_legacy_numbers() {
        let stored: StoredValue = serde_json::from_str("42").unwrap();
        let restored = stored.to_value();

        assert_eq!(formatter::format_value(&restored), "42");
    }
}
