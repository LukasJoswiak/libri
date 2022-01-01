use std::collections::HashMap;
use std::error::Error;

use crate::common;
use crate::config;
use crate::format::epub;

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let mut ebooks = HashMap::new();
    let ebook_paths = common::find_books(&config.library)?;

    // TODO: Abstract this part as well, also used in import
    for path in ebook_paths {
        let ebook = match epub::parse(path.as_path()) {
            Ok(ebook) => ebook,
            Err(e) => return Err(e),
        };

        let entry = ebooks.entry(ebook.author).or_insert(Vec::new());
        entry.push(ebook.title);
    }

    // TODO: Sort alphabetically by author, then by title
    for (author, title) in ebooks {
        println!("{}\n  {}", author, title.join("\n  "));
    }

    Ok(())
}
