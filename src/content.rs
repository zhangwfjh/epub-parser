#[derive(Debug, Clone, Default)]
pub struct Page {
    pub index: usize,
    pub content: String,
}

impl Page {
    pub fn new(index: usize, content: String) -> Self {
        Page { index, content }
    }
}
