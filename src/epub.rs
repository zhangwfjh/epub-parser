use crate::types::{Image, Metadata, Page, TocEntry};
use crate::utils::{preprocess_html_entities, ZipHandler};
use ordered_hash_map::OrderedHashMap;
use quick_xml::events::Event;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// A parsed EPUB e-book representation.
///
/// This struct contains all the extracted data from an EPUB file including:
/// - Metadata (title, author, publisher, etc.)
/// - Table of contents (hierarchical navigation)
/// - Text content (pages in reading order)
/// - Images (including cover image)
///
/// # Example
///
/// ```
/// use epub_parser::Epub;
/// use std::path::Path;
///
/// let epub = Epub::parse(Path::new("book.epub"))?;
///
/// // Access metadata
/// if let Some(title) = &epub.metadata.title {
///     println!("Title: {}", title);
/// }
///
/// // Access all pages
/// for page in &epub.pages {
///     println!("Page {}: {} chars", page.index, page.content.len());
/// }
///
/// // Access images
/// for image in &epub.images {
///     println!("Image: {} ({})", image.href, image.media_type);
/// }
/// ```
#[derive(Debug)]
pub struct Epub {
    /// The Dublin Core metadata extracted from the EPUB.
    pub metadata: Metadata,
    /// The hierarchical table of contents from the NCX file.
    pub toc: Vec<TocEntry>,
    /// The text content of each page in reading order.
    pub pages: Vec<Page>,
    /// All images found in the EPUB (including cover).
    pub images: Vec<Image>,
}

/// Errors that can occur while parsing an EPUB file.
#[derive(Debug)]
pub enum Error {
    /// The EPUB file is invalid or corrupted.
    InvalidEpub(String),
    /// An I/O error occurred while reading the file.
    IoError(std::io::Error),
    /// An error occurred while reading the ZIP archive.
    ZipError(zip::result::ZipError),
    /// An error occurred while parsing XML.
    XmlError(String),
    /// The META-INF/container.xml file is missing.
    MissingContainer,
    /// The OPF package file is missing or not found.
    MissingOpf,
    /// The NCX table of contents file is missing.
    MissingNcx,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidEpub(msg) => write!(f, "Invalid EPUB: {}", msg),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
            Error::ZipError(e) => write!(f, "ZIP error: {}", e),
            Error::XmlError(e) => write!(f, "XML error: {}", e),
            Error::MissingContainer => write!(f, "Missing container.xml"),
            Error::MissingOpf => write!(f, "Missing OPF file"),
            Error::MissingNcx => write!(f, "Missing NCX file"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            Error::ZipError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        Error::ZipError(err)
    }
}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Error::XmlError(err.to_string())
    }
}

impl Epub {
    /// Parse an EPUB file from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the EPUB file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Epub)` on success, or an `Error` if parsing fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file does not exist
    /// - The file is not a valid ZIP archive
    /// - The EPUB structure is invalid
    ///
    /// # Example
    ///
    /// ```
    /// use epub_parser::Epub;
    /// use std::path::Path;
    ///
    /// let epub = Epub::parse(Path::new("book.epub"))?;
    /// println!("Parsed: {}", epub.metadata.title.unwrap_or_default());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(path: &Path) -> Result<Self, Error> {
        let mut zip_handler = ZipHandler::new(path)?;
        Self::parse_from_handler(&mut zip_handler)
    }

    /// Parse an EPUB file from a byte buffer.
    ///
    /// This is useful when you have the EPUB data in memory, for example
    /// when downloading from a network or reading from a database.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The raw bytes of the EPUB file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Epub)` on success, or an `Error` if parsing fails.
    ///
    /// # Example
    ///
    /// ```
    /// use epub_parser::Epub;
    ///
    /// let bytes = std::fs::read("book.epub")?;
    /// let epub = Epub::parse_from_buffer(&bytes)?;
    /// println!("Parsed: {}", epub.metadata.title.unwrap_or_default());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_from_buffer(buffer: &[u8]) -> Result<Self, Error> {
        let cursor = Cursor::new(buffer.to_vec());
        let mut zip_handler = ZipHandler::new_from_reader(cursor)?;
        Self::parse_from_handler(&mut zip_handler)
    }

    fn parse_from_handler<R: std::io::Read + std::io::Seek>(
        zip_handler: &mut ZipHandler<R>,
    ) -> Result<Self, Error> {
        let opf_path = zip_handler.get_opf_path()?;
        let opf_content = zip_handler.read_file(&opf_path)?;

        let (metadata, manifest, spine, ncx_path) = Self::parse_opf(&opf_content)?;

        let toc = if let Some(ncx_ref) = ncx_path {
            let ncx_path_full = Self::resolve_path(&opf_path, &ncx_ref);
            let ncx_content = zip_handler.read_file(&ncx_path_full)?;
            Self::parse_ncx(&ncx_content)?
        } else {
            Vec::new()
        };

        let mut pages = Vec::new();
        for itemref in spine {
            if let Some(manifest_item) = manifest.get(&itemref) {
                let content_path = Self::resolve_path(&opf_path, &manifest_item.href);
                let content = zip_handler.read_file(&content_path)?;
                let text = Self::extract_text_from_html(&content)?;
                pages.push(Page {
                    index: pages.len(),
                    content: text,
                });
            }
        }

        let mut images = Vec::new();
        for (id, item) in &manifest {
            if item._media_type.to_lowercase().starts_with("image/") {
                let image_path = Self::resolve_path(&opf_path, &item.href);
                if let Ok(bytes) = zip_handler.read_file_as_bytes(&image_path) {
                    if id.to_lowercase().contains("cover") {
                        images.insert(
                            0,
                            Image {
                                id: id.clone(),
                                href: item.href.clone(),
                                media_type: item._media_type.clone(),
                                content: bytes,
                            },
                        );
                    } else {
                        images.push(Image {
                            id: id.clone(),
                            href: item.href.clone(),
                            media_type: item._media_type.clone(),
                            content: bytes,
                        });
                    }
                }
            }
        }

        Ok(Epub {
            metadata,
            toc,
            pages,
            images,
        })
    }

    fn parse_opf(
        content: &str,
    ) -> Result<
        (
            Metadata,
            OrderedHashMap<String, ManifestItem>,
            Vec<String>,
            Option<String>,
        ),
        Error,
    > {
        let content = preprocess_html_entities(content);
        let mut reader = quick_xml::Reader::from_str(&content);
        let mut metadata = Metadata::new();
        let mut manifest: OrderedHashMap<String, ManifestItem> = OrderedHashMap::new();
        let mut spine: Vec<String> = Vec::new();
        let mut ncx_path: Option<String> = None;

        let mut current_text_tag: Option<String> = None;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name.contains("title") {
                        current_text_tag = Some("title".to_string());
                    } else if name.contains("creator") {
                        current_text_tag = Some("author".to_string());
                    } else if name.contains("publisher") {
                        current_text_tag = Some("publisher".to_string());
                    } else if name.contains("language") {
                        current_text_tag = Some("language".to_string());
                    } else if name.contains("identifier") {
                        current_text_tag = Some("identifier".to_string());
                    } else if name.contains("date") {
                        current_text_tag = Some("date".to_string());
                    } else if name.contains("rights") {
                        current_text_tag = Some("rights".to_string());
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name.contains("item") && !name.contains("itemref") {
                        let mut id = String::new();
                        let mut href = String::new();
                        let mut media_type = String::new();

                        for attr_result in e.attributes() {
                            if let Ok(attr) = attr_result {
                                let attr_name =
                                    String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if attr_name == "id" || attr_name.ends_with(":id") {
                                    if let Some(val) =
                                        attr.decode_and_unescape_value(reader.decoder()).ok()
                                    {
                                        id = val.to_string();
                                    }
                                } else if attr_name == "href" || attr_name.ends_with(":href") {
                                    href = attr
                                        .decode_and_unescape_value(reader.decoder())?
                                        .to_string();
                                } else if attr_name == "media-type"
                                    || attr_name.ends_with(":media-type")
                                {
                                    media_type = attr
                                        .decode_and_unescape_value(reader.decoder())?
                                        .to_string();
                                }
                            }
                        }

                        if !id.is_empty() && !href.is_empty() {
                            if media_type == "application/x-dtbncx+xml" {
                                ncx_path = Some(href.clone());
                            }
                            manifest.insert(
                                id.clone(),
                                ManifestItem {
                                    _id: id.clone(),
                                    href,
                                    _media_type: media_type,
                                },
                            );
                        }
                    } else if name.contains("itemref") {
                        let mut idref = String::new();

                        for attr_result in e.attributes() {
                            if let Ok(attr) = attr_result {
                                let attr_name =
                                    String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if attr_name == "idref" || attr_name.ends_with(":idref") {
                                    if let Some(val) =
                                        attr.decode_and_unescape_value(reader.decoder()).ok()
                                    {
                                        idref = val.to_string();
                                    }
                                    break;
                                }
                            }
                        }

                        if !idref.is_empty() {
                            spine.push(idref);
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some(tag) = &current_text_tag {
                        let text = e.unescape()?.into_owned().trim().to_string();
                        if !text.is_empty() {
                            match tag.as_str() {
                                "title" => metadata.title = Some(text),
                                "author" => metadata.author = Some(text),
                                "publisher" => metadata.publisher = Some(text),
                                "language" => metadata.language = Some(text),
                                "identifier" => metadata.identifier = Some(text),
                                "date" => metadata.date = Some(text),
                                "rights" => metadata.rights = Some(text),
                                _ => {}
                            }
                        }
                        current_text_tag = None;
                    }
                }
                Ok(Event::End(_)) => {
                    current_text_tag = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::XmlError(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok((metadata, manifest, spine, ncx_path))
    }

    fn parse_ncx(content: &str) -> Result<Vec<TocEntry>, Error> {
        let content = preprocess_html_entities(content);
        let mut reader = quick_xml::Reader::from_str(&content);
        let mut toc = Vec::new();
        let mut stack: Vec<TocEntry> = Vec::new();

        let mut buf = Vec::new();
        let mut in_nav_label = false;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "navPoint" {
                        let entry = TocEntry {
                            label: String::new(),
                            href: String::new(),
                            children: Vec::new(),
                        };
                        stack.push(entry);
                    } else if name == "navLabel" {
                        in_nav_label = true;
                    } else if name == "text" && in_nav_label {
                        in_text = true;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "navPoint" {
                        if let Some(entry) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(entry);
                            } else {
                                toc.push(entry);
                            }
                        }
                    } else if name == "navLabel" {
                        in_nav_label = false;
                    } else if name == "text" && in_nav_label {
                        in_text = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_text {
                        if let Some(entry) = stack.last_mut() {
                            entry.label = e.unescape()?.into_owned();
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "content" {
                        if let Some(src) = e.try_get_attribute("src")? {
                            if let Some(entry) = stack.last_mut() {
                                entry.href =
                                    src.decode_and_unescape_value(reader.decoder())?.to_string();
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::XmlError(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(toc)
    }

    fn extract_text_from_html(content: &str) -> Result<String, Error> {
        let content = preprocess_html_entities(content);
        let mut reader = quick_xml::Reader::from_str(&content);
        let mut text = String::new();
        let skip_tags: Vec<Vec<u8>> = vec![b"script".to_vec(), b"style".to_vec(), b"head".to_vec()];
        let mut in_skip_tag = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = e.name().as_ref().to_vec();
                    if skip_tags.contains(&tag) {
                        in_skip_tag = true;
                    } else if tag.as_slice() == b"p"
                        || tag.as_slice() == b"div"
                        || tag.as_slice() == b"br"
                        || tag.as_slice() == b"li"
                    {
                        text.push('\n');
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag = e.name().as_ref().to_vec();
                    if skip_tags.contains(&tag) {
                        in_skip_tag = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if !in_skip_tag {
                        let t = e.unescape()?.into_owned();
                        let trimmed: String = t.chars().filter(|c| !c.is_control()).collect();
                        text.push_str(&trimmed);
                        text.push(' ');
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::XmlError(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n"))
    }

    fn resolve_path(base_path: &str, href: &str) -> String {
        let base = PathBuf::from(base_path);
        let parent = base.parent().unwrap_or(base.as_path());
        let resolved = parent.join(href);
        resolved.to_string_lossy().to_string().replace('\\', "/")
    }
}

#[derive(Debug, Clone)]
struct ManifestItem {
    _id: String,
    href: String,
    _media_type: String,
}
