//! A Rust library for parsing EPUB e-book files.
//!
//! This library provides functionality to extract metadata, table of contents,
//! text content, and images from EPUB files. It follows the EPUB specification
//! and parses both OPF and NCX files to provide a complete representation
//! of the e-book's structure.
//!
//! # Features
//!
//! - Parse EPUB container and locate OPF file
//! - Extract Dublin Core metadata (title, author, publisher, language, etc.)
//! - Parse NCX table of contents with hierarchical structure
//! - Extract text from HTML/XHTML content files
//! - Extract cover image and all images from EPUB
//! - Follow reading order from OPF spine
//! - Clean text extraction (strips HTML, handles line breaks)
//!
//! # Example
//!
//! ```
//! use epub_parser::Epub;
//! use std::path::Path;
//!
//! let epub = Epub::parse(Path::new("book.epub"))?;
//!
//! // Access metadata
//! if let Some(title) = &epub.metadata.title {
//!     println!("Title: {}", title);
//! }
//!
//! // Access table of contents
//! for entry in &epub.toc {
//!     println!("- {} ({})", entry.label, entry.href);
//! }
//!
//! // Access page content
//! for page in &epub.pages {
//!     println!("Page {}: {} characters", page.index, page.content.len());
//! }
//!
//! // Access images
//! for image in &epub.images {
//!     println!("Image: {} ({})", image.href, image.media_type);
//! }
//! ```

pub mod epub;
pub mod types;
pub mod utils;

pub use epub::Epub;
pub use types::{Image, Metadata, Page, TocEntry};
pub use utils::{XmlParser, ZipHandler};
