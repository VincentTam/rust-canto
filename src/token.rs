use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Token {
    pub word: String,
    #[serde(rename = "jyutping")]
    pub reading: Option<String>,
    pub yale: Option<Vec<String>>,
}
