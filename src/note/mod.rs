use regex::Regex;
use std::collections::BTreeSet;
use open;

#[derive(PartialEq, Eq, PartialOrd, Ord, RustcEncodable, RustcDecodable)]
struct Note {
  id: usize,
  title: String,
  tags: BTreeSet<String>,
  body: String
}

impl Note {
  fn open(&self) -> () {
    if !open::that(AsRef::<std::ffi::OsStr>::as_ref(&self.body)).is_ok() {
      println!("Failed to open note.");
    }
  }

  fn filter_id(&self, filter: String) -> bool {
    filter.parse::<usize>()
      .ok()
      .map(|id| id == self.id)
      .unwrap_or(false)
  }

  fn filter_title(&self, filter: String) -> bool {
    Regex::new(&filter)
      .ok()
      .map(|re| re.is_match(&self.title))
      .unwrap_or(false)
  }

  fn filter_tags(&self, filter: String) -> bool {
    parse_tags(filter).is_subset(&self.tags)
  }

  fn filter_body(&self, filter: String) -> bool {
    filter == self.body
  }
}

fn parse_tags(comma_delimited: String) -> BTreeSet<String> {
  comma_delimited
    .split(",")
    .map(|t| String::from(t.trim()))
    .filter(|t| t.len() > 0)
    .collect()
}