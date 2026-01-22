use crate::epub::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub struct ZipHandler {
    archive: ZipArchive<File>,
}

impl ZipHandler {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;
        Ok(ZipHandler { archive })
    }

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

    pub fn read_file(&mut self, path: &str) -> Result<String, Error> {
        let mut file = self.archive.by_name(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    pub fn read_file_as_bytes(&mut self, path: &str) -> Result<Vec<u8>, Error> {
        let mut file = self.archive.by_name(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }
}
