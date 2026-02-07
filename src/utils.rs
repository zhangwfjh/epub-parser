//! Utility modules for EPUB parsing.
//!
//! This module provides helper utilities for parsing EPUB files:
//! - `ZipHandler`: For reading EPUB ZIP archives
//! - `XmlParser`: For extracting text from XML elements

pub mod xml;
pub mod zip;

pub use xml::{preprocess_html_entities, XmlParser};
pub use zip::ZipHandler;
