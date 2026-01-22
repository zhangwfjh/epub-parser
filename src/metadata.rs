#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub date: Option<String>,
    pub rights: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }
}
