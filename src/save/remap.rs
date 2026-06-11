use std::{collections::HashMap};

use serde::{Deserialize, Deserializer};
use ree_lib::data::{DataSource, deserialize_data_sources};

#[derive(Deserialize, Debug, Clone)]
pub struct Remap {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_format_string")]
    pub format: Vec<FormatNode>,
    // remaps a field from one type to another
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_data_sources")]
    pub queries: HashMap<String, DataSource>, 
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_data_sources")]
    pub data: HashMap<String, DataSource>, 
}

#[derive(Debug, Clone)]
pub enum FormatNode {
    Literal(String),        // "Enemy:"
    Data(String),           // "{data:name}" -> "name"
    Convert(String),        // "{convert:app.EnemyDef.ID_Fixed}" -> "app.EnemyDef.ID_Fixed"
    Enum,                   // "{enum:}"
    Field(String),          // "{_BasicData}" -> "_BasicData"
}

use regex::Regex;

pub fn parse_format_string(format_str: &str) -> Vec<FormatNode> {
    let mut nodes = Vec::new();

    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let mut last_end = 0;

    for cap in re.captures_iter(format_str) {
        let whole_match = cap.get(0).unwrap();
        let inner_text = cap.get(1).unwrap().as_str();

        if whole_match.start() > last_end {
            let literal = &format_str[last_end..whole_match.start()];
            nodes.push(FormatNode::Literal(literal.to_string()));
        }

        let parts: Vec<&str> = inner_text.splitn(2, ':').collect();
        let command = parts[0].trim();
        let arg = if parts.len() > 1 { parts[1].trim() } else { "" };

        match command {
            "data" => nodes.push(FormatNode::Data(arg.to_string())),
            "convert" => nodes.push(FormatNode::Convert(arg.to_string())),
            "enum" => nodes.push(FormatNode::Enum),
            _ if arg.is_empty() => nodes.push(FormatNode::Field(command.to_string())),
            _ => nodes.push(FormatNode::Literal(whole_match.as_str().to_string())),
        }

        last_end = whole_match.end();
    }

    if last_end < format_str.len() {
        nodes.push(FormatNode::Literal(format_str[last_end..].to_string()));
    }

    nodes
}

fn deserialize_format_string<'de, D>(deserializer: D) -> Result<Vec<FormatNode>, D::Error>
where
    D: Deserializer<'de>,
{
    let format_str = String::deserialize(deserializer)?;
    Ok(parse_format_string(&format_str))
}
