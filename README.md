# libri

Libri is a command line ebook manager.

Libri organizes and stores your ebook collection with a standardized naming convention parsed from the metadata of each book. The project is currently in early development and compatibility between releases is not guaranteed.

Sample libri session:

```
$ tree ~/Desktop/ebooks  # a collection of ebooks downloaded from gutenberg.org
/Users/lukasjoswiak/Desktop/ebooks
├── pg1400.epub
├── pg2701.epub
├── pg345-images.epub
├── pg64317-images.epub
└── pg98.epub

$ libri config
Config { library: "/Users/lukasjoswiak/Documents/books/" }

$ libri import ~/Desktop/ebooks/
imported "Dracula"
imported "Great Expectations"
imported "The Great Gatsby"
imported "A Tale of Two Cities"
imported "Moby Dick; Or, The Whale"

imported 5; skipped 0; finished in 0.05s

$ libri list
Title                     Author               Created
––––––––––––––––––––––––  –––––––––––––––––––  ––––––––––––––––
A Tale of Two Cities      Charles Dickens      January 01, 2022
Great Expectations        Charles Dickens      January 01, 2022
Dracula                   Bram Stoker          January 01, 2022
Moby Dick; Or, The Whale  Herman Melville      January 01, 2022
The Great Gatsby          F. Scott Fitzgerald  January 01, 2022

$ tree ~/Documents/books
/Users/lukasjoswiak/Documents/books
├── Bram Stoker
│   └── Dracula
│       └── Dracula.epub
├── Charles Dickens
│   ├── A Tale of Two Cities
│   │   └── A Tale of Two Cities.epub
│   └── Great Expectations
│       └── Great Expectations.epub
├── F. Scott Fitzgerald
│   └── The Great Gatsby
│       └── The Great Gatsby.epub
└── Herman Melville
    └── Moby Dick; Or, The Whale
        └── Moby Dick; Or, The Whale.epub
```

## Install

The best way to use libri is to build from source. Clone the project, then run `cargo build`. A binary will be generated at `target/debug/libri`.

The default folder libri stores imported ebooks in is `$HOME/Documents/books/`. To modify the default location, create the file `$HOME/.config/libri/config.ini`, and set `library` to the path you want ebooks saved to:

```
library = /Users/lukasjoswiak/books
```

Tilde expansion is not yet supported, so make sure to use an absolute path for now. Run `libri config` to make sure libri is correctly reading your updated configuration.
