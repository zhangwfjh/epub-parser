//! Type definitions for EPUB book components.
//!
//! This module contains all the data structures used to represent
//! the extracted content from an EPUB file:
//! - `Metadata`: Dublin Core metadata
//! - `Page`: Text content pages
//! - `Image`: Images including cover
//! - `TocEntry`: Table of contents navigation

/// Dublin Core metadata extracted from an EPUB file.
///
/// This struct contains standard Dublin Core metadata fields as defined
/// in the EPUB specification. All fields are optional as not all EPUBs
/// contain complete metadata.
///
/// # Example
///
/// ```
/// use epub_parser::Metadata;
///
/// let metadata = Metadata {
///     title: Some("Example Book".to_string()),
///     author: Some("John Doe".to_string()),
///     publisher: Some("Example Press".to_string()),
///     language: Some("en".to_string()),
///     ..Default::default()
/// };
///
/// println!("Book: {}", metadata.title.unwrap_or_default());
/// ```
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// The title of the book.
    ///
    /// Maps to the Dublin Core `dc:title` element.
    pub title: Option<String>,

    /// The author or creator of the book.
    ///
    /// Maps to the Dublin Core `dc:creator` element.
    pub author: Option<String>,

    /// The publisher of the book.
    ///
    /// Maps to the Dublin Core `dc:publisher` element.
    pub publisher: Option<String>,

    /// The language code (e.g., "en", "fr", "zh").
    ///
    /// Maps to the Dublin Core `dc:language` element.
    pub language: Option<String>,

    /// A unique identifier for the book (e.g., ISBN, UUID).
    ///
    /// Maps to the Dublin Core `dc:identifier` element.
    pub identifier: Option<String>,

    /// The publication date.
    ///
    /// Maps to the Dublin Core `dc:date` element.
    /// Typically in YYYY-MM-DD format.
    pub date: Option<String>,

    /// The copyright or rights statement.
    ///
    /// Maps to the Dublin Core `dc:rights` element.
    pub rights: Option<String>,
}

impl Metadata {
    /// Creates a new, empty Metadata instance.
    ///
    /// # Returns
    ///
    /// A `Metadata` struct with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A single page of text content from an EPUB book.
///
/// Pages are extracted from the EPUB's HTML/XHTML content files in the
/// order defined by the spine element in the OPF file. Each page
/// contains the plain text content with HTML tags stripped.
///
/// # Example
///
/// ```
/// use epub_parser::Page;
///
/// let page = Page::new(0, "Chapter 1\n\nIt was a dark and stormy night...".to_string());
/// println!("Page {}: {} characters", page.index, page.content.len());
/// ```
#[derive(Debug, Clone, Default)]
pub struct Page {
    /// The position of this page in the reading order (0-indexed).
    pub index: usize,

    /// The plain text content of the page.
    ///
    /// HTML tags are stripped, and the text is cleaned of control characters.
    /// Paragraphs and other block elements are separated by newlines.
    pub content: String,
}

impl Page {
    /// Creates a new Page with the given index and content.
    ///
    /// # Arguments
    ///
    /// * `index` - The position in the reading order.
    /// * `content` - The plain text content of the page.
    ///
    /// # Returns
    ///
    /// A new `Page` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use epub_parser::Page;
    ///
    /// let page = Page::new(5, "Some text content".to_string());
    /// assert_eq!(page.index, 5);
    /// ```
    pub fn new(index: usize, content: String) -> Self {
        Page { index, content }
    }
}

/// An image extracted from an EPUB file.
///
/// Images include both the metadata (ID, href, media type) and optionally
/// the binary content. The first image in the EPUB's images vector is
/// typically the cover image.
///
/// # Example
///
/// ```
/// use epub_parser::Image;
///
/// let image = Image {
///     id: "cover".to_string(),
///     href: "images/cover.jpg".to_string(),
///     media_type: "image/jpeg".to_string(),
///     content: None,
/// };
///
/// println!("Image: {} ({})", image.href, image.media_type);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Image {
    /// The unique identifier for this image from the manifest.
    ///
    /// This corresponds to the `id` attribute in the OPF manifest.
    pub id: String,

    /// The path to the image within the EPUB archive.
    ///
    /// This is a relative path that can be used to locate the image file
    /// within the EPUB's ZIP structure.
    pub href: String,

    /// The MIME type of the image.
    ///
    /// Common values include "image/jpeg", "image/png", "image/gif", etc.
    pub media_type: String,

    /// The binary content of the image.
    ///
    /// This is the raw bytes of the image file.
    pub content: Vec<u8>,
}

/// An entry in the EPUB table of contents.
///
/// EPUB navigation uses NCX (Navigation Control for XML) files which define
/// a hierarchical structure of navigation points. Each entry can have
/// child entries, creating a tree structure.
///
/// # Example
///
/// ```
/// use epub_parser::TocEntry;
///
/// let mut toc_entry = TocEntry::new("Chapter 1".to_string(), "chapter1.xhtml".to_string());
/// toc_entry.children.push(TocEntry::new(
///     "Section 1.1".to_string(),
///     "chapter1.xhtml#section1".to_string()
/// ));
///
/// println!("Entry: {} -> {}", toc_entry.label, toc_entry.href);
/// for child in &toc_entry.children {
///     println!("  Child: {} -> {}", child.label, child.href);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct TocEntry {
    /// The display label or title for this navigation point.
    ///
    /// This is the text that would be shown in a table of contents.
    pub label: String,

    /// The target URL for this navigation point.
    ///
    /// This is a relative path within the EPUB, often with an anchor
    /// (e.g., "chapter1.xhtml" or "chapter1.xhtml#section1").
    pub href: String,

    /// Child navigation entries.
    ///
    /// The NCX format supports hierarchical navigation, so each entry
    /// can have nested sub-entries.
    pub children: Vec<TocEntry>,
}

impl TocEntry {
    /// Creates a new TOC entry with the given label and href.
    ///
    /// The children vector is initialized as empty.
    ///
    /// # Arguments
    ///
    /// * `label` - The display text for this entry.
    /// * `href` - The target URL/path for this entry.
    ///
    /// # Returns
    ///
    /// A new `TocEntry` instance with empty children.
    ///
    /// # Example
    ///
    /// ```
    /// use epub_parser::TocEntry;
    ///
    /// let entry = TocEntry::new("Introduction".to_string(), "intro.xhtml".to_string());
    /// assert_eq!(entry.label, "Introduction");
    /// assert_eq!(entry.href, "intro.xhtml");
    /// assert!(entry.children.is_empty());
    /// ```
    pub fn new(label: String, href: String) -> Self {
        TocEntry {
            label,
            href,
            children: Vec::new(),
        }
    }
}
