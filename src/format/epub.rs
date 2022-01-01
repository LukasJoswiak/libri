use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use zip::{result, ZipArchive};

use crate::xml;
use crate::Ebook;

pub fn parse(path: &Path) -> Result<Ebook, Box<dyn Error>> {
    let mut archive = ZipArchive::new(File::open(path)?)?;
    let metadata_path = get_metadata_path(&mut archive)?;
    let document = parse_metadata(&mut archive, metadata_path.as_path())?;

    Ok(Ebook::new(
        document
            .elements
            .iter()
            .find(|x| x.prefix.as_deref() == Some("dc") && x.tag == "title")
            .unwrap()
            .content
            .clone(),
        document
            .elements
            .iter()
            .find(|x| x.prefix.as_deref() == Some("dc") && x.tag == "creator")
            .unwrap()
            .content
            .clone(),
        path,
    ))
}

fn get_metadata_path<R: Read + Seek>(archive: &mut ZipArchive<R>) -> result::ZipResult<PathBuf> {
    let mut container = archive.by_name("META-INF/container.xml")?;

    // TODO: Add tests for this case
    if container.enclosed_name() == None {
        panic!("failed to read epub metadata");
    }

    let mut contents = String::new();
    container.read_to_string(&mut contents)?;

    let document = match xml::parse(&contents) {
        Ok(document) => document,
        // TODO: Add tests for this case
        Err(error) => panic!("a problem occurred while parsing metadata: {}", error),
    };
    let element = document.elements.iter().find(|x| x.tag == "rootfile");

    match element {
        Some(element) => {
            match element.attributes.get("full-path") {
                Some(path) => Ok(PathBuf::from(path)),
                // TODO: Add tests for this case
                None => panic!("rootfile tag missing full-path attribute"),
            }
        }
        None => {
            // TODO: Add tests for this case
            panic!("epub contains improperly formatted metadata file")
        }
    }
}

fn parse_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    metadata_path: &Path,
) -> result::ZipResult<xml::XmlDocument> {
    let mut metadata = archive.by_name(metadata_path.to_str().unwrap())?;

    if metadata.enclosed_name() == None {
        panic!("failed to read epub data");
    }

    let mut contents = String::new();
    metadata.read_to_string(&mut contents)?;
    match xml::parse(&contents) {
        Ok(document) => Ok(document),
        // TODO: Add tests for this case
        Err(error) => panic!("a problem occurred while parsing the book: {}", error),
    }
}
