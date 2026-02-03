//! ZIP archive handling for EPUB files.
//!
//! This module provides utilities for reading EPUB files, which are
//! ZIP archives containing XML, HTML, and media files. It handles
//! locating the OPF file via META-INF/container.xml and reading
//! files from the archive.

use crate::epub::Error;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use zip::ZipArchive;

/// A handler for reading EPUB files as ZIP archives.
///
/// This struct wraps a `ZipArchive` and provides convenience methods
/// for reading specific files needed for EPUB parsing:
/// - Locating the OPF file via META-INF/container.xml
/// - Reading text files (XML, HTML, CSS, etc.)
/// - Reading binary files (images, fonts, etc.)
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use epub_parser::utils::ZipHandler;
///
/// let mut handler = ZipHandler::new(Path::new("book.epub"))?;
/// let opf_path = handler.get_opf_path()?;
/// println!("OPF location: {}", opf_path);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ZipHandler<R: Read + Seek> {
    archive: ZipArchive<R>,
}

impl ZipHandler<File> {
    /// Creates a new ZipHandler from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the EPUB file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ZipHandler)` on success, or an error if the file
    /// cannot be opened or is not a valid ZIP archive.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The file cannot be opened
    /// - The file is not a valid ZIP archive
    pub fn new(path: &Path) -> Result<Self, Error> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;
        Ok(ZipHandler { archive })
    }
}

impl<R: Read + Seek> ZipHandler<R> {
    /// Creates a new ZipHandler from any reader that implements
    /// `Read + Seek`.
    ///
    /// This is useful for parsing EPUBs from memory (e.g., byte buffers)
    /// or network streams.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any type implementing `Read + Seek` (e.g., `Cursor<Vec<u8>>`).
    ///
    /// # Returns
    ///
    /// Returns `Ok(ZipHandler)` on success, or an error if the reader
    /// does not contain a valid ZIP archive.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use epub_parser::utils::ZipHandler;
    ///
    /// let data = vec![0u8; 100]; // In practice, this would be EPUB data
    /// // handler = ZipHandler::new_from_reader(Cursor::new(data))?;
    /// ```
    pub fn new_from_reader(reader: R) -> Result<Self, Error> {
        let archive = ZipArchive::new(reader)?;
        Ok(ZipHandler { archive })
    }

    /// Locates the OPF (Open Package Format) file path.
    ///
    /// EPUB files contain a `META-INF/container.xml` file that specifies
    /// the location of the OPF file. This method parses that XML and
    /// returns the path to the OPF file.
    ///
    /// # Returns
    ///
    /// Returns the path to the OPF file as a string (e.g., "OEBPS/content.opf").
    ///
    /// # Errors
    ///
    /// Returns `Error::MissingContainer` if META-INF/container.xml is missing.
    /// Returns `Error::MissingOpf` if the OPF path cannot be found.
    pub fn get_opf_path(&mut self) -> Result<String, Error> {
        let container_content = self.read_file("META-INF/container.xml")?;

        let mut reader = quick_xml::Reader::from_str(&container_content);
        let mut opf_path = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if name == "rootfile" || name.ends_with(":rootfile") {
                        for attr_result in e.attributes() {
                            if let Ok(attr) = attr_result {
                                let attr_name =
                                    String::from_utf8_lossy(attr.key.as_ref()).to_string();

                                if attr_name == "full-path" || attr_name.ends_with(":full-path") {
                                    opf_path = attr
                                        .decode_and_unescape_value(reader.decoder())?
                                        .to_string();
                                    break;
                                }
                            }
                        }
                        if !opf_path.is_empty() {
                            break;
                        }
                    }
                }
                Ok(quick_xml::events::Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if name == "rootfile" || name.ends_with(":rootfile") {
                        for attr_result in e.attributes() {
                            if let Ok(attr) = attr_result {
                                let attr_name =
                                    String::from_utf8_lossy(attr.key.as_ref()).to_string();

                                if attr_name == "full-path" || attr_name.ends_with(":full-path") {
                                    opf_path = attr
                                        .decode_and_unescape_value(reader.decoder())?
                                        .to_string();
                                    break;
                                }
                            }
                        }
                        if !opf_path.is_empty() {
                            break;
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(Error::XmlError(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        if opf_path.is_empty() {
            return Err(Error::MissingOpf);
        }

        Ok(opf_path)
    }

    /// Reads a file from the ZIP archive as a UTF-8 string.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file within the ZIP archive.
    ///
    /// # Returns
    ///
    /// Returns the file contents as a `String`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist in the archive
    /// - The file cannot be read
    /// - The file contains invalid UTF-8
    pub fn read_file(&mut self, path: &str) -> Result<String, Error> {
        let mut file = self.archive.by_name(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Reads a file from the ZIP archive as raw bytes.
    ///
    /// This is useful for binary files like images and fonts.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file within the ZIP archive.
    ///
    /// # Returns
    ///
    /// Returns the file contents as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist or cannot be read.
    pub fn read_file_as_bytes(&mut self, path: &str) -> Result<Vec<u8>, Error> {
        let mut file = self.archive.by_name(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }
}
