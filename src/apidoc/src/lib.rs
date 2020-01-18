use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl Serialize for HttpMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        })
    }
}

impl<'de> Deserialize<'de> for HttpMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => unimplemented!(),
        })
    }
}

/// An apiDoc route.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    #[serde(rename = "type")]
    pub method: HttpMethod,
    pub url: String,
    pub name: String,
    pub group: String,
    pub version: String,
    pub parameter: Option<ParameterSection>,
    pub success: Option<ParameterSection>,
    pub error: Option<ParameterSection>,
    pub filename: String,
    pub group_title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterSection {
    pub fields: HashMap<String, Vec<Parameter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub group: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub optional: bool,
    pub field: String,
    pub description: String,
}
