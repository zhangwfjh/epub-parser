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

let epub = Epub::parse(Path::new("book.epub"))?;

// Access metadata
println!("Title: {:?}", epub.metadata.title);
println!("Author: {:?}", epub.metadata.author);

// Access cover image
if let Some(ref href) = epub.cover.href {
    println!("Cover: {}", href);
    if let Some(ref content) = epub.cover.content {
        println!("Cover size: {} bytes", content.len());
        // Save cover image
        std::fs::write("cover.jpg", content)?;
    }
}

// Access images
for image in &epub.images {
    println!("Image: {} ({} bytes)", image.href, 
        image.content.as_ref().map(|c| c.len()).unwrap_or(0));
    if let Some(ref content) = image.content {
        std::fs::write(&format!("images/{}", image.href), content)?;
    }
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
