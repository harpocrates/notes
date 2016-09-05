use bincode::rustc_serialize::{encode_into, decode_from};
// use getopts::Options;
use std::fs::File;
use std::collections::BTreeMap;
use std::env;
use bincode::SizeLimit;
use std::path::Path;
use rand;
use clap::ArgMatches;

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

// Load notes from cache
pub fn load_from_default_cache() -> Option<BTreeMap<usize,note::Note>> {
  env::home_dir()
    .and_then(|mut home| { home.push(".notes-cache"); load_from_cache(home.as_path()) })
}

// Write notes to cache
pub fn write_to_default_cache(existing: &BTreeMap<usize,note::Note>) -> Option<()> {
  env::home_dir()
    .and_then(|mut home| { home.push(".notes-cache"); write_to_cache(existing, home.as_path()) })
}


pub fn open_list_notes(matches: &ArgMatches, open: bool) -> () {

  // load the matching notes from the cache
  let cache = match load_from_default_cache() {
    None => { println!("No cache of notes exists yet."); return }
    Some(cache) => cache
  };

  let mut matching = cache
    .values()
    .filter(|note| -> bool 
      { matches.value_of("title").map_or(true, |title| note.filter_title(title))
      & matches.values_of("body").map_or(true, |mut bodies| bodies.any(|body| note.filter_body(body)))
      & matches.values_of("id").map_or(true, |mut ids| ids.any(|id| note.filter_id(id)))
      & matches.values_of("tags").map_or(true, |tags| note.filter_tags(tags.map(String::from).collect())) });

  let num_display = matches
    .value_of("lines")
    .and_then(|s| s.parse::<usize>().ok())
    .unwrap_or(10);
  
  // print the matching notes
  println!("Note UUID  | Title                | Tags                    | Body            ");
  println!("-----------+----------------------+-------------------------+-----------------");

  for note in matching.by_ref().take(num_display) {
    println!("{:010} | {: <20} | {: <23} | {: <17}",
      note.id,
      note.title,
      note.tags.iter().fold(String::new(), |s,tag| if s!="" { s+" "+ &tag } else { s+&tag }),
      note.body);

    if open {
      note.open();
    }
  }

  // remind the user about remaining undisplayed/unopened notes (if there are any)
  let remaining = matching.count();
  if remaining > 0 {
    println!("There are {} matching notes not {}.", remaining, if open { "opened" } else { "listed" });
  }
}


pub fn new_note(matches: &ArgMatches) -> () {

  // create the note
  let note = note::Note { id: rand::random()
                        , title: matches.value_of("title").map(String::from).unwrap()
                        , tags: matches.values_of("tags").map(|tags| tags.map(String::from).collect()).unwrap_or_default()
                        , body: matches.value_of("body").map(String::from).unwrap()
                        };

  // Update the cache
  let mut cache = load_from_default_cache().unwrap_or(BTreeMap::new());
  cache.insert(note.id, note);
  match write_to_default_cache(&cache) {
    None => println!("Failed to write note."),
    Some(_) => println!("Note written. There are now {} notes.", cache.len()),
  }
}


pub fn update_note(matches: &ArgMatches) -> () {

  // load the cache
  let mut cache = load_from_default_cache().unwrap_or(BTreeMap::new());

  // find and remove from the cache the note
  let note = matches
    .value_of("id")
    .and_then(|s| s.parse::<usize>().ok())
    .and_then(|id| cache.remove(&id));

  // create the updated note and insert it into the cache
  match note {
    None => { println!("Invalid id {}.", matches.value_of("id").unwrap()); return }
    Some(old_note) => {
      let new_note = note::Note { id: old_note.id
                                , title: matches.value_of("title").map(String::from).unwrap_or(old_note.title)
                                , tags: matches.values_of("tags").map(|tags| tags.map(String::from).collect()).unwrap_or(old_note.tags)
                                , body: matches.value_of("body").map(String::from).unwrap_or(old_note.body)
                                };

      cache.insert(new_note.id, new_note);
    }
  }
  
  // save the updated cache
  match write_to_default_cache(&cache) {
    None => println!("Failed to update note."),
    Some(_) => println!("Note updated."),
  }
}