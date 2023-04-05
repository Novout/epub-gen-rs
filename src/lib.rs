use std::fs;
use std::io::{Cursor, Write};
use zip::write::{FileOptions};
use slugify::slugify;
use chrono::{Local};
use uuid::Uuid;

pub struct Info {
  pub title: String,
  pub description: String,
  pub publisher: String,
  pub author: String,
  pub toc_title: String,
  pub lang: String,
  pub fonts: Vec<String>,
  pub css: Option<String>,
  pub version: i8,
}

pub struct EPUB {
  info: Info,
  chapters: Vec<Vec<String>>
}

impl EPUB {
  pub fn new(info: Info, chapters: Vec<Vec<String>>) -> EPUB {
    EPUB {
      info,
      chapters
    }
  }

  pub fn run(&mut self) {
    let archive_result = self.archive();

    let archive: Vec<u8> = match archive_result {
      Ok(vec) => vec,
      Err(err) => panic!("{}", err)
    };

    self.write(archive);
  }

  fn write_chapters(&self) -> Vec<(&String, String)> {
    let mut _chapters = vec![];

    for chapter in &self.chapters {
      let title = &chapter[0];
      let content = chapter
        .iter()
        .skip(1)
        .map(|raw| format!("<p>{}</p>", raw))
        .reduce(|cur: String, nxt: String| cur + &nxt + "\n")
        .unwrap();

      let template = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE html>
<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" xml:lang=\"{lang}\" lang=\"{lang}\">
  <head>
    <meta charset=\"UTF-8\" />
    <title>{title}</title>
    <link rel=\"stylesheet\" type=\"text/css\" href=\"styles.css\" />
  </head>
  <body>
    <h1>{title}</h1>
    {content}
  </body>
</html>", title=self.info.title, lang=self.info.lang, content=content);

      _chapters.push((title, template));
    } 

    _chapters
  }

  fn manifest(&self) -> String {
    let xhtml_targets: String = self.chapters
      .iter()
      .map(|s| format!("<item id=\"{}\" href=\"{}.xhtml\" media-type=\"application/xhtml+xml\" />", slugify!(&s[0]), slugify!(&s[0], separator = "_")))
      .reduce(|cur: String, nxt: String| cur + &nxt + "\n")
      .unwrap();

    format!("<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\" />
<item id=\"toc\" href=\"toc.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\" />
<item id=\"css\" href=\"styles.css\" media-type=\"text/css\" />
{}", xhtml_targets)
  }

  fn toc_xhtml(&self) -> String {
    let mut li = vec![];

    for chapter in self.chapters.iter() {
      let title = &chapter[0];
      let src = format!("{}.xhtml", slugify!(title, separator = "_"));

      li.push(format!("<li class=\"table-of-content\">
      <a href=\"{}\">{}</a>
    </li>", src, title));
    }

    li
      .iter()
      .map(|s| s.to_string())
      .reduce(|cur: String, nxt: String| cur + &nxt + "\n")
      .unwrap()
  }

  fn toc_ncx(&self) -> String {
    let mut li = vec![];

    for (index, chapter) in self.chapters.iter().enumerate() {
      let next = index + 1;
      let content_id = format!("content_{}_item_{}", index, index);
      let title = format!("{}. {}", next, &chapter[0]);
      let src = format!("{}.xhtml", slugify!(&chapter[0], separator = "_"));

      li.push(format!("<navPoint id=\"{}\" playOrder=\"{}\" class=\"chapter\">
  <navLabel>
    <text>{}</text>
  </navLabel>
  <content src=\"{}\"/>
</navPoint>", content_id, next, title, src));
    }

    li
      .iter()
      .map(|s| s.to_string())
      .reduce(|cur: String, nxt: String| cur + &nxt + "\n")
      .unwrap()
  }
  
  pub fn archive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut archive = Cursor::new(Vec::new());

    let mut zip = zip::ZipWriter::new(&mut archive);

    let stored = FileOptions::default()
      .compression_method(zip::CompressionMethod::Stored)
      .unix_permissions(0o755);

    // write raw for one-generation in toc.(ncx|xhtml) and *.xhtml
    let chapters = self.write_chapters();

    // mimetype
    zip.start_file("mimetype", stored)?;
    zip.write_all(b"application/epub+zip")?;

    // META_INF
    zip.add_directory("META_INF/", stored)?;
    zip.start_file("META_INF/container.xml", stored)?;
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>
<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">
  <rootfiles>
    <rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>
  </rootfiles>
</container>")?;

    // OEBPS
    zip.add_directory("OEBPS/", Default::default())?;
    // content.opf (info and file bindings)
    zip.start_file("OEBPS/content.opf", Default::default())?;
    // uuid for unique-identifier
    let unique_identifier: Uuid = Uuid::new_v4();
    let content: String = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
  <package 
    xmlns=\"http://www.idpf.org/2007/opf\"
    version=\"3.0\"
    unique-identifier=\"BookId\"
    xmlns:dc=\"http://purl.org/dc/elements/1.1/\"
    xmlns:dcterms=\"http://purl.org/dc/terms/\"
    xml:lang=\"{lang}\"
    xmlns:media=\"http://www.idpf.org/epub/vocab/overlays/#\"
    prefix=\"ibooks: http://vocabulary.itunes.apple.com/rdf/ibooks/vocabulary-extensions-1.0/\">

  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:opf=\"http://www.idpf.org/2007/opf\">
    <dc:identifier id=\"BookId\">{uuid}</dc:identifier>
    <meta refines=\"#BookId\" property=\"identifier-type\" scheme=\"onix:codelist5\">22</meta>
    <meta property=\"dcterms:identifier\" id=\"meta-identifier\">BookId</meta>
    <dc:title>{title}</dc:title>
    <meta property=\"dcterms:title\" id=\"meta-title\">{title}</meta>
    <dc:language>{lang}</dc:language>
    <meta property=\"dcterms:language\" id=\"meta-language\">{lang}</meta>
    <meta property=\"dcterms:modified\">{date}</meta>
    <dc:creator id=\"creator\">{author}</dc:creator>
    <meta refines=\"#creator\" property=\"file-as\">{author}</meta>
    <meta property=\"dcterms:publisher\">{publisher}</meta>
    <dc:publisher>{publisher}</dc:publisher>
    <meta property=\"dcterms:date\">{date}</meta>
    <dc:date>{date}</dc:date>
    <meta property=\"dcterms:rights\">All rights reserved</meta>
    <dc:rights>Copyright &#x00A9; 2023 by {publisher}</dc:rights>
    <meta name=\"generator\" content=\"epub-gen-rs\" />
    <meta property=\"ibooks:specified-fonts\">false</meta>
  </metadata>
  <manifest>
    {manifest}
  </manifest>
</package>", uuid=unique_identifier, author=self.info.author, lang=self.info.lang, title=self.info.title, date=Local::now(), publisher=self.info.publisher, manifest=self.manifest());

    zip.write_all(content.as_bytes())?;

    // toc.ncx (navigator)
    zip.start_file("OEBPS/toc.ncx", Default::default())?;
    let toc = format!("
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\">
  <head>
    <meta name=\"dtb:uid\" content=\"{uuid}\" />
    <meta name=\"dtb:generator\" content=\"epub-gen-rs\"/>
    <meta name=\"dtb:depth\" content=\"1\"/>
    <meta name=\"dtb:totalPageCount\" content=\"0\"/>
    <meta name=\"dtb:maxPageNumber\" content=\"0\"/>
  </head>
  <docTitle>
    <text>{title}</text>
  </docTitle>
  <docAuthor>
    <text>{author}</text>
  </docAuthor>
  <navMap>
    <navPoint id=\"toc\" playOrder=\"0\" class=\"chapter\">
      <navLabel>
        <text>{toc_title}</text>
      </navLabel>
      <content src=\"toc.xhtml\"/>
    </navPoint>
    {toc}
  </navMap>
</ncx>", uuid=unique_identifier, author=self.info.author, title=self.info.title, toc_title=self.info.toc_title, toc=self.toc_ncx());

    zip.write_all(toc.as_bytes())?;

    // toc.xhtml (anchor)
    zip.start_file("OEBPS/toc.xhtml", Default::default())?;
    let toc = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE html>
<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" xml:lang=\"{lang}\" lang=\"{lang}\">
<head>
    <title>{title}</title>
    <meta charset=\"UTF-8\" />
    <link rel=\"stylesheet\" type=\"text/css\" href=\"styles.css\" />
</head>
<body>
  <h1 class=\"h1\">Table of Content</h1>
  <nav id=\"toc\" epub:type=\"toc\">
    <ol>
      {toc}
    </ol>
  </nav>
</body>
</html>", lang=self.info.lang, title=self.info.title, toc=self.toc_xhtml());

    zip.write_all(toc.as_bytes())?;

    // XHTML's
    for (title, raw) in chapters.iter() {
      zip.start_file(format!("OEBPS/{}.xhtml", slugify!(title, separator = "_")), stored)?;
      zip.write_all(raw.as_bytes())?;
    }

    // CSS
    zip.start_file("OEBPS/styles.css", Default::default())?;
    match &self.info.css {
      Some(css) => {
        zip.write_all(css.as_bytes())?;
      },
      None => {
        zip.write_all(b"")?;
      }
    }

    Ok(zip.finish().unwrap().clone().into_inner())
  }

  pub fn write(&mut self, data: Vec<u8>)  {
    fs::write(&format!("{}.epub", &self.info.title), data).ok();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! items {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
  }  

  #[test]
  fn it_build() {
    let mut epub = EPUB::new(Info {
      title: String::from("test"),
      description: String::from("test"),
      publisher: String::from("test"),
      author: String::from("test"),
      toc_title: String::from("test"),
      lang: String::from("en"),
      fonts: vec![String::from("en")],
      css: None,
      version: 3
    }, vec![items![
      "Title",
      "Nullam tempor, metus vitae sagittis semper, massa nulla posuere ipsum, nec mollis tortor dui sed enim. Praesent ac orci posuere, iaculis elit at, eleifend lorem.",
      "Aliquam non posuere ex. Duis fermentum odio metus, quis ultrices nulla cursus vitae. Nullam blandit, nisi non posuere volutpat, lorem lorem aliquet ex, eu sagittis turpis felis nec dui. Integer iaculis arcu vitae elementum convallis. Pellentesque tempor, eros eu consectetur cursus, magna turpis lacinia nunc, ut pulvinar velit est non mauris. Nunc at erat purus. Morbi at arcu libero. Sed ac lobortis erat, id egestas tellus. Nullam velit turpis, maximus eget lacus quis, fringilla rhoncus odio. Praesent quam magna, maximus sed ullamcorper quis, dictum at turpis."
    ]]);
    
    epub.run();
  }
}
