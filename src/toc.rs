#[derive(Debug, Clone, Default)]
pub struct TocEntry {
    pub label: String,
    pub href: String,
    pub children: Vec<TocEntry>,
}

impl TocEntry {
    pub fn new(label: String, href: String) -> Self {
        TocEntry {
            label,
            href,
            children: Vec::new(),
        }
    }
}
