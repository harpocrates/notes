use regex::Regex;
use std::collections::BTreeSet;
use open;
use std::ffi;

#[derive(PartialEq, Eq, PartialOrd, Ord, RustcEncodable, RustcDecodable)]
pub struct Note {
  pub id: usize,
  pub title: String,
  pub tags: BTreeSet<String>,
  pub body: String
}

impl Note {
  pub fn open(&self) -> () {
    if !open::that(AsRef::<ffi::OsStr>::as_ref(&self.body)).is_ok() {
      println!("Failed to open note.");
    }
  }

  pub fn filter_id(&self, filter: String) -> bool {
    filter.parse::<usize>()
      .ok()
      .map(|id| id == self.id)
      .unwrap_or(false)
  }

  pub fn filter_title(&self, filter: String) -> bool {
    Regex::new(&filter)
      .ok()
      .map(|re| re.is_match(&self.title))
      .unwrap_or(false)
  }

  pub fn filter_tags(&self, filter: String) -> bool {
    parse_tags(filter).is_subset(&self.tags)
  }

  pub fn filter_body(&self, filter: String) -> bool {
    filter == self.body
  }
}

pub fn parse_tags(comma_delimited: String) -> BTreeSet<String> {
  comma_delimited
    .split(",")
    .map(|t| String::from(t.trim()))
    .filter(|t| t.len() > 0)
    .collect()
}