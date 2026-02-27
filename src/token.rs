use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Token {
    pub word: String,
    pub reading: Option<String>,
}
