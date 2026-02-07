//! Utility modules for EPUB parsing.
//!
//! This module provides helper utilities for parsing EPUB files:
//! - `ZipHandler`: For reading EPUB ZIP archives
//! - `XmlParser`: For extracting text from XML elements

pub mod xml;
pub mod zip;

pub use xml::{XmlParser, preprocess_html_entities};
pub use zip::ZipHandler;
