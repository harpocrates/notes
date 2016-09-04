use bincode::rustc_serialize::{encode_into, decode_from};
use getopts::Options;
use std::fs::File;
use std::collections::BTreeMap;
use std::env;
use bincode::SizeLimit;
use std::path::Path;
use rand;

use note;

// Load notes from cache
pub fn load_from_cache(location: &Path) -> Option<BTreeMap<usize,note::Note>> {
  File::open(location)
    .ok()
    .and_then(|mut file| decode_from(&mut file, SizeLimit::Infinite).ok())
}

// Write notes to cache
pub fn write_to_cache(existing: &BTreeMap<usize,note::Note>, location: &Path) -> Option<()> {
  File::create(location)
    .ok()
    .and_then(|mut file| encode_into(existing, &mut file, SizeLimit::Infinite).ok())
}


pub fn open_list_notes(args: Vec<String>, list_or_open: bool) -> () {

  // Options for finding notes
  let mut opts = Options::new();
  opts.optopt("t","title", "regex to match to title of the note", "TITLE");
  opts.optopt("b", "body", "note with exactly the given body path", "FILE");
  opts.optopt("i", "id", "note with exactly the given id", "INT");
  opts.optopt("a", "tags", "note having all given tags, comma delimited", "TAGS");
  opts.optopt("n","number", "limit for number of notes, defaults to 10", "INT");

  let matches = match opts.parse(&args[2..]) {
    Ok(m) => m,
    Err(f) => {
      println!("{}", f.to_string());
      print!("{}", opts.usage("notes open/find"));
      return
    }
  };

  // Load cache
  let cache = match env::home_dir() {
    None => { println!("Can't find home directory!"); return }
    Some(mut home) => {
      home.push(".notes-cache");
      let path = home.as_path();

      load_from_cache(path).unwrap_or(BTreeMap::new())
    }
  };

  // get matching notes
  let matches = cache
    .values()
    .filter(|note| -> bool 
      { matches.opt_str("title").map_or(true, |title| note.filter_title(title))
      & matches.opt_str("body").map_or(true, |body| note.filter_body(body))
      & matches.opt_str("id").map_or(true, |id| note.filter_id(id))
      & matches.opt_str("tags").map_or(true, |tags| note.filter_tags(tags)) })
    .take(matches.opt_str("number").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10));

  
  if list_or_open {
    println!("Note UUID  | Title                | Tags                    | Body            ");
    println!("-----------+----------------------+-------------------------+-----------------");
    for note in matches {
      println!("{:010} | {: <20} | {: <23} | {: <17}",
        note.id,
        note.title,
        note.tags.iter().fold(String::new(), |s,tag| if s!="" { s+" "+ &tag } else { s+&tag }),
        note.body);
    }
  } else {
    for note in matches {
      note.open();
    }
  }; 
}


pub fn new_note(args: Vec<String>) -> () {

  // Options for creating a note
  let mut opts = Options::new();
  opts.reqopt("t","title", "title of the note", "TITLE");
  opts.optopt("a","tags", "tags for the note, comma delimited", "TAGS");
  opts.reqopt("b", "body", "set path to body of note", "FILE");

  let matches = match opts.parse(&args[2..]) {
    Ok(m) => m,
    Err(f) => {
      println!("{}", f.to_string());
      print!("{}", opts.usage("notes new"));
      return
    }
  };

  // create the note
  let note = note::Note { id: rand::random()
                        , title: matches.opt_str("title").unwrap()
                        , tags: matches.opt_str("tags").map(note::parse_tags).unwrap_or_default()
                        , body: matches.opt_str("body").unwrap()
                        };

  // Update the cache
  match env::home_dir() {
    None => { println!("Can't find home directory!"); return }
    Some(mut home) => {
      home.push(".notes-cache");
      let path = home.as_path();

      let mut cache = load_from_cache(path).unwrap_or(BTreeMap::new());
      cache.insert(note.id, note);
      write_to_cache(&cache, path);

      println!("{}", cache.len())
    }
  }
  
    
}