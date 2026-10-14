use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum NameValue {
    String(String),
    Formatted(FormattedValue),
}
impl NameValue {
    pub(crate) fn get_name(&self) -> &str {
        match self {
            NameValue::String(s) => &*s,
            NameValue::Formatted(v) => &*v.value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormattedValue {
    #[serde(rename = "v")]
    pub(crate) value: String,
    #[serde(rename = "f")]
    pub(crate) formatting: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "RowAsList", into = "RowAsList")]
pub struct Row {
    pub(crate) name: NameValue,
    pub(crate) parent: Option<String>,
    pub(crate) tooltip: Option<String>,
}
impl Row {
    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.into());
        self
    }
}
impl From<RowAsList> for Row {
    fn from(value: RowAsList) -> Self {
        Self {
            name: value.0,
            parent: value.1,
            tooltip: value.2,
        }
    }
}

pub type Rows = Vec<Row>;

#[derive(Debug, Serialize, Deserialize)]
struct RowAsList(NameValue, Option<String>, Option<String>);
impl From<Row> for RowAsList {
    fn from(value: Row) -> Self {
        Self(value.name, value.parent, value.tooltip)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn a() {
        let a: Rows = Vec::from([
            Row {
                name: NameValue::Formatted(FormattedValue {
                    value: String::from("meow"),
                    formatting: String::from("<h1>MEOW</h1>"),
                }),
                parent: None,
                tooltip: Some(String::from("purr")),
            },
            Row {
                name: NameValue::String(String::from("purr")),
                parent: Some(String::from("meow")),
                tooltip: None,
            },
        ]);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            String::from(
                "[[{\"v\":\"meow\",\"f\":\"<h1>MEOW</h1>\"},null,\"purr\"],[\"purr\",\"meow\",null]]"
            )
        )
    }
}
