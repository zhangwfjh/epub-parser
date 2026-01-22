#[derive(Debug, Clone, Default)]
pub struct Image {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub content: Option<Vec<u8>>,
}
