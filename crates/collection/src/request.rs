use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Http(HttpRequest),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,

    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<Vec<u8>>,
    #[serde(default)]
    pub query: Option<Vec<(String, String)>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}
