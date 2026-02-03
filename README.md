# epub-parser

A Rust library for extracting metadata, table of contents, text, cover, and images from EPUB files

## Features

- ✅ Parse EPUB container and locate OPF file
- ✅ Extract Dublin Core metadata (title, author, publisher, language, identifier, date, rights)
- ✅ Parse NCX table of contents with hierarchical structure
- ✅ Extract text from HTML/XHTML content files
- ✅ Extract cover image from EPUB
- ✅ Extract all images from EPUB
- ✅ Follow reading order from OPF spine
- ✅ Clean text extraction (strips HTML, handles line breaks)

## Dependencies

- `zip` - for extracting EPUB (which is a ZIP archive)
- `quick-xml` - for parsing XML (OPF, NCX) and HTML content

## Usage

```rust
use epub_parser::Epub;
use std::path::Path;

// Parse from file path
let epub = Epub::parse(Path::new("book.epub"))?;

// Or parse from in-memory buffer
let buffer = std::fs::read("book.epub")?;
let epub_from_buffer = Epub::parse_from_buffer(&buffer)?;

// Access metadata
println!("Title: {:?}", epub.metadata.title);
println!("Author: {:?}", epub.metadata.author);

// Access images, the first is cover
for image in &epub.images {
    println!("Image: {} ({} bytes)", image.href, image.content.len());
    std::fs::write(&format!("{}", image.href), image.content.clone())?;
}

// Access table of contents
for entry in &epub.toc {
    println!("- {} ({})", entry.label, entry.href);
}

// Access page content
for page in &epub.pages {
    println!("Page {}: {} characters", page.index, page.content.len());
}
```
