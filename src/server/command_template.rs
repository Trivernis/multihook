use jsonpath::Selector;
use lazy_static::lazy_static;
use regex::{Match, Regex};
use serde_json::Value;

#[derive(Clone)]
pub struct CommandTemplate {
    src: String,
    matches: Vec<(usize, usize)>,
}

impl CommandTemplate {
    pub fn new<S: ToString>(command: S) -> Self {
        lazy_static! {
            static ref PLACEHOLDER_REGEX: Regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
        }
        let command = command.to_string();
        let matches = PLACEHOLDER_REGEX
            .find_iter(&command)
            .map(|m: Match| (m.start(), m.end()))
            .collect();
        Self {
            src: command,
            matches,
        }
    }

    pub fn evaluate(&self, json: &Value) -> String {
        let mut result_string = String::with_capacity(self.src.len());
        let mut last_index = 0;

        for (start, end) in &self.matches {
            let before = &self.src[last_index..*start];
            let query = &self.src[*start + 2..*end - 2];
            result_string.push_str(before);
            result_string.push_str(&evaluate_path(query, json).unwrap_or_default());

            last_index = *end;
        }
        result_string.push_str(&self.src[last_index..]);

        result_string
    }
}

fn evaluate_path(query: &str, json: &Value) -> Option<String> {
    let selector = Selector::new(query).ok()?;
    let results = selector
        .find(json)
        .map(json_to_string)
        .collect::<Vec<String>>();

    Some(results.join("\n"))
}

fn json_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::with_capacity(0),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_owned(),
        Value::Array(a) => a
            .iter()
            .map(|v| json_to_string(v))
            .collect::<Vec<String>>()
            .join("\n"),
        Value::Object(o) => o
            .iter()
            .map(|(k, v)| format!("{} = {}", k, json_to_string(v)))
            .collect::<Vec<String>>()
            .join("\n"),
    }
}
