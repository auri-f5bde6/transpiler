use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum NameValue<T: Into<String>> {
    String(String),
    Formatted(FormattedValue<T>),
}
impl<T: Into<String>> NameValue<T> {
    pub(crate) fn get_name(&self) -> &str {
        match self {
            NameValue::String(s) => &*s,
            NameValue::Formatted(v) => &*v.value,
        }
    }
    pub(crate) fn map<
        F1: FnOnce(String) -> String,
        F2: FnOnce(FormattedValue<T>) -> FormattedValue<T>,
    >(
        self,
        f1: F1,
        f2: F2,
    ) -> Self {
        match self {
            NameValue::String(s) => NameValue::String(f1(s)),
            NameValue::Formatted(f) => NameValue::Formatted(f2(f)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormattedValue<T: Into<String>> {
    #[serde(rename = "v")]
    pub(crate) value: String,
    #[serde(rename = "f")]
    pub(crate) formatting: T,
}
impl<T: Into<String>> FormattedValue<T> {
    pub(crate) fn map_formatting<F: FnOnce(T) -> T>(mut self, f: F) -> Self {
        self.formatting = f(self.formatting);
        self
    }
}
#[derive(Debug, Serialize, Clone)]
#[serde(into = "String")]
pub struct AstDisplay {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
}
impl AstDisplay {
    fn escape_html(html: &str) -> String {
        html.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
            .replace("'", "&#39;")
    }
    pub(crate) fn with_subtitle(mut self, subtitle: String) -> Self {
        self.map_subtitle(|_| Some(subtitle))
    }
    pub(crate) fn map_subtitle<F: FnOnce(Option<String>) -> Option<String>>(
        mut self,
        f: F,
    ) -> Self {
        self.subtitle = f(self.subtitle);
        self
    }
}
impl Into<String> for AstDisplay {
    fn into(self) -> String {
        format!(
            "<div class='ast-name'>{}</div>{}",
            Self::escape_html(&*self.title),
            self.subtitle.map_or(String::new(), |x| format!(
                "<div class='ast-value'>{}</div>",
                Self::escape_html(&*x)
            ))
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "RowAsList<T>", into = "RowAsList<T>")]
pub struct Row<T: Into<String> + std::clone::Clone> {
    pub(crate) name: NameValue<T>,
    pub(crate) parent: Option<String>,
    pub(crate) tooltip: Option<String>,
}
impl<T: Into<String> + std::clone::Clone> Row<T> {
    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.into());
        self
    }
    pub fn map_name<F: FnOnce(NameValue<T>) -> NameValue<T>>(mut self, f: F) -> Self {
        self.name = f(self.name);
        self
    }
    pub fn map_display_name<F1: FnOnce(String) -> String, F2: FnOnce(T) -> T>(
        self,
        f1: F1,
        f2: F2,
    ) -> Self {
        self.map_name(|name| name.map(f1, |x| x.map_formatting(f2)))
    }
}
impl<T: Into<String> + std::clone::Clone> From<RowAsList<T>> for Row<T> {
    fn from(value: RowAsList<T>) -> Self {
        Self {
            name: value.0,
            parent: value.1,
            tooltip: value.2,
        }
    }
}

pub type Rows<T> = Vec<Row<T>>;

#[derive(Debug, Serialize, Deserialize)]
struct RowAsList<T: Into<String>>(NameValue<T>, Option<String>, Option<String>);
impl<T: Into<String> + std::clone::Clone> From<Row<T>> for RowAsList<T> {
    fn from(value: Row<T>) -> Self {
        Self(value.name, value.parent, value.tooltip)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn a() {
        let a: Rows<String> = Vec::from([
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
