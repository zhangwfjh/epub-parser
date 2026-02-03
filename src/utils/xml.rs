//! XML parsing utilities for the EPUB parser.
//!
//! This module provides helper functions for common XML parsing tasks,
//! particularly for extracting text content from XML elements.

use quick_xml::events::Event;

/// A utility for parsing XML content.
///
/// This struct provides helper methods for common XML parsing operations
/// used when extracting data from EPUB files (OPF, NCX, and HTML content).
pub struct XmlParser;

impl XmlParser {
    /// Extracts text content from an XML reader.
    ///
    /// Reads events from the XML reader until a text event is found or
    /// the element ends. This is useful for extracting the text content
    /// of XML elements like `<title>`, `<creator>`, etc.
    ///
    /// # Arguments
    ///
    /// * `reader` - The XML reader to read events from.
    /// * `buf` - A buffer for reading events (will be cleared automatically).
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(String))` if text was found, `Ok(None)` if no
    /// text was found before the element ended, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an XML parsing error.
    ///
    /// # Example
    ///
    /// ```
    /// use quick_xml::Reader;
    /// use epub_parser::utils::XmlParser;
    ///
    /// let xml = r#"<title>My Book</title>"#;
    /// let mut reader = Reader::from_str(xml);
    /// let mut buf = Vec::new();
    ///
    /// // Skip the Start event
    /// reader.read_event_into(&mut buf).unwrap();
    ///
    /// let text = XmlParser::extract_text(&mut reader, &mut buf).unwrap();
    /// assert_eq!(text, Some("My Book".to_string()));
    /// ```
    pub fn extract_text<R: std::io::BufRead>(
        reader: &mut quick_xml::Reader<R>,
        buf: &mut Vec<u8>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut text = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Text(e)) => {
                    text = e.unescape()?.into_owned();
                    text = text.trim().to_string();
                    if !text.is_empty() {
                        break;
                    }
                }
                Ok(Event::End(_)) => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(if text.is_empty() { None } else { Some(text) })
    }
}
