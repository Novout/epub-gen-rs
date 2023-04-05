# EPUB Implementation in Rust (WIP)

[https://www.w3.org/TR/epub-33/](https://www.w3.org/TR/epub-33/)

- [ ] Stable
- [x] Base Header and Text Flow
- [x] ToC
- [x] Deflated Zip
- [x] Stored for valid decrypted files
- [x] UUID for XHTML's items
- [ ] UUID for resources
- [ ] UUID for paths
- [ ] Validade data entries
- [ ] Optional META_INF (encryption.xml, metadata.xml, manifest.xml, ...)
- [ ] `container.xml` access
- [ ] Images, Checkbox and List's
- [ ] Custom Fonts

### Example

```rs
let mut epub = EPUB::new(Info {
  title: String::from("A Nice Title"),
  description: String::from("A some description..."),
  publisher: String::from("..."),
  author: String::from("..."),
  toc_title: String::from("Table of Contents"),
  append_chapter_titles: false,
  date: String::from("2323-02-02"),
  lang: String::from("en"),
  fonts: vec![String::from("Roboto")],
  css: None,
  version: 3
}, vec![macro_for_this_please![
  "Title",
  "A some content...",
]]);

epub.run();
```