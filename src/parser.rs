use quick_xml::events::Event;

pub struct XmlParser;

impl XmlParser {
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
