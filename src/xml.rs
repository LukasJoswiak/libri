use std::collections::HashMap;

use xml::reader::{self, XmlEvent};

#[derive(PartialEq, Debug)]
pub struct XmlDocument {
    // TODO: Other metadata
    pub encoding: String,
    pub elements: Vec<XmlElement>,
}

impl XmlDocument {
    fn new() -> XmlDocument {
        XmlDocument {
            encoding: String::new(),
            elements: Vec::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct XmlElement {
    pub prefix: Option<String>,
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
}

pub fn parse(data: &str) -> reader::Result<XmlDocument> {
    let mut document = XmlDocument::new();

    for event in xml::reader::EventReader::from_str(data) {
        if let Ok(ev) = event {
            match ev {
                XmlEvent::StartDocument {
                    version: _,
                    encoding,
                    standalone: _,
                } => {
                    document.encoding = encoding;
                }
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    let mut attrs = HashMap::new();
                    for attr in attributes {
                        attrs.insert(attr.name.local_name, attr.value);
                    }
                    document.elements.push(XmlElement {
                        prefix: name.prefix,
                        tag: name.local_name,
                        attributes: attrs,
                        content: String::new(),
                    });
                }
                XmlEvent::Characters(c) => {
                    if let Some(element) = document.elements.last_mut() {
                        element.content = c;
                    }
                }
                _ => (),
            }
        }
    }

    Ok(document)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::xml;

    #[test]
    fn parse_simple_xml() {
        let xml = "<test attr='value'>contents</test>";
        let mut attributes = HashMap::new();
        attributes.insert(String::from("attr"), String::from("value"));
        let document = xml::parse(xml).unwrap();
        assert_eq!(1, document.elements.len());
        assert_eq!(
            xml::XmlElement {
                prefix: None,
                tag: String::from("test"),
                attributes: attributes,
                content: String::from("contents"),
            },
            *document.elements.get(0).unwrap()
        );
    }
}
