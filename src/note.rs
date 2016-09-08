use regex::Regex;
use std::collections::BTreeSet;
use open;
use std::ffi;
use std::fs::canonicalize;
use std::path::Path;
use std::fmt::Display;


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, RustcEncodable, RustcDecodable)]
pub struct Note {
  pub id: usize,
  pub title: String,
  pub tags: BTreeSet<String>,
  pub body: String,
}

impl Note {
  pub fn open(&self) -> () {
    if !open::that(AsRef::<ffi::OsStr>::as_ref(&self.body)).is_ok() {
      println!("Failed to open note.");
    }
  }

  pub fn filter_id(&self, filter: &str) -> bool {
    usize::from_str_radix(filter,16)
      .ok()
      .map(|id| id == self.id)
      .unwrap_or(false)
  }

  pub fn filter_title(&self, filter: &str) -> bool {
    Regex::new(filter)
      .ok()
      .map(|re| re.is_match(&self.title))
      .unwrap_or(false)
  }

  pub fn filter_tags(&self, filter: BTreeSet<String>) -> bool {
    filter.is_subset(&self.tags)
  }

  pub fn filter_body(&self, filter: &str) -> bool {
    filter == &self.body
  }
}

pub fn sanitize_path<P: AsRef<Path> + Display + Clone>(path: P) -> Result<String,String> {
  canonicalize(path.as_ref())
    .map_err(|_| format!("cannot canonicalize path {}", path.clone()))
    .map(|path| String::from(path.to_str().unwrap()))
}
