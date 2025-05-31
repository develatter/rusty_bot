use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDocumentResult {
    pub title: String,
    pub body: String,
    pub score: f32,
}

impl SimpleDocumentResult {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}