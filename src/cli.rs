use bincode::rustc_serialize::{encode_into, decode_from};
use bincode::SizeLimit;
use clap::ArgMatches;
use rand;
use rustc_serialize::json;

use std::collections::{BTreeSet, BTreeMap};
use std::env;
use std::fs::{File, canonicalize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use note;
use error::Error;

// Load notes from cache
pub fn load_from_cache(location: &Path) -> Result<BTreeMap<usize,note::Note>,Error> {
  let mut file = try!(File::open(location).map_err(|_| Error::OpenCache));
  decode_from(&mut file, SizeLimit::Infinite)
    .map_err(|_| Error::DecodeCache)
}

// Write notes to cache
pub fn write_to_cache(existing: &BTreeMap<usize,note::Note>, location: &Path) -> Result<(),Error> {
  let mut file = try!(File::create(location).map_err(|_| Error::CreateCache));
  encode_into(existing, &mut file, SizeLimit::Infinite)
    .map_err(|_| Error::EncodeCache)
}

// Load notes from cache
pub fn load_from_default_cache() -> Result<BTreeMap<usize,note::Note>,Error> {
  let mut home = try!(env::home_dir().ok_or(Error::HomeDir));
  home.push(".notes-cache");
  load_from_cache(home.as_path())
}

// Write notes to cache
pub fn write_to_default_cache(existing: &BTreeMap<usize,note::Note>) -> Result<(),Error> {
  let mut home = try!(env::home_dir().ok_or(Error::HomeDir));
  home.push(".notes-cache");
  write_to_cache(existing, home.as_path())
}


pub fn open_list_notes(matches: &ArgMatches, open: bool) -> Result<(),Error> {

  // load the matching notes from the cache
  let cache = try!(load_from_default_cache());

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
  
  let long = matches.is_present("long");

  // print the matching notes
  if open || !long {
    println!("Note UUID        | Title                  | Tags                               ");
    println!("-----------------+------------------------+------------------------------------");
  } else {
    println!("Notes found:");
    println!("-------------")
  }

  for note in matching.by_ref().take(num_display) {
    
    let tags = note.tags.iter().fold(String::new(), |s,tag| if s!="" { s+" "+ &tag } else { s+&tag });

    if open || !long {
      println!("{:016X} | {: <23.23}| {: <35.35}", note.id, note.title, tags);
    } else {
      println!("  ID:    {:016X}", note.id);
      println!("  Title: {}", note.title);
      println!("  Tags:  {}", tags);
      println!("  Body:  {}", note.body);
      println!("-------------")
    }

    if open {
      note.open();
    }
  }

  // remind the user about remaining undisplayed/unopened notes (if there are any)
  let remaining = matching.count();
  if remaining > 0 {
    println!("{} matching note(s) was not {}.", remaining, if open { "opened" } else { "listed" });
  }

  Ok(())
}


pub fn drop_notes(matches: &ArgMatches) -> Result<(),Error> {

  fn prompt_user(note: &note::Note) -> bool {
    use std::io::{stdin,stdout,Write};

    println!("Are you sure you want to delete note '{}' [{:016X}]", note.title, note.id);
    let _ = stdout().flush();

    loop {
      let mut input = String::new();
      let _ = stdin().read_line(&mut input);
      
      if input.starts_with("y") || input.starts_with("yes") {
        return true;
      } else if input.starts_with("n") || input.starts_with("no") {
        return false;
      } else {
        println!("Invalid response. Expecting 'yes' or 'no'.");
      }
    };
  }

  // load the matching notes from the cache
  let old_cache = try!(load_from_default_cache());

  let new_cache = old_cache
    .into_iter()
    .filter(|&(_, ref note)| -> bool {
        ( !matches.value_of("title").map_or(true, |title| note.filter_title(title))
        || !matches.values_of("body").map_or(true, |mut bodies| bodies.any(|body| note.filter_body(body)))
        || !matches.values_of("id").map_or(true, |mut ids| ids.any(|id| note.filter_id(id)))
        || !matches.values_of("tags").map_or(true, |tags| note.filter_tags(tags.map(String::from).collect())))
        || !(matches.is_present("force") || prompt_user(&note)) 
    })
    .collect();

  // save the updated cache
  try!(write_to_default_cache(&new_cache));
  println!("Note cache updated.");

  Ok(())
}



pub fn new_note(matches: &ArgMatches) -> Result<(),Error> {

  // Check path is valid
  let body = try!(note::sanitize_path(matches.value_of("body").unwrap()));

  let title = matches
    .value_of("title")
    .map(String::from);
  
  let tags: Option<BTreeSet<String>> = matches
        .values_of("tags")
        .map(|tags| tags.map(String::from).collect());

  // create the note
  let note = note::Note { id: rand::random()
                        , title: title.unwrap()
                        , tags: tags.unwrap_or_default()
                        , body: body
                        };

  // Update the cache
  let mut cache = load_from_default_cache().ok().unwrap_or_default();
  cache.insert(note.id, note);
  try!(write_to_default_cache(&cache));

  println!("Note written. There are now {} notes.", cache.len());

  Ok(())
}


pub fn update_note(matches: &ArgMatches) -> Result<(),Error> {

  // load the cache
  let mut cache = try!(load_from_default_cache());

  // find and remove from the cache the note
  let old_note = try!(usize::from_str_radix(matches.value_of("id").unwrap(),16)
    .map_err(|_| Error::MalformedId)
    .and_then(|id| cache.remove(&id).ok_or(Error::NoSuchNote(id))));

  // create the updated note and insert it into the cache
  let body: Option<String> = match matches.value_of("body") {
    None => None,
    Some(body) => Some(try!(note::sanitize_path(body))),
  };
  
  let title: Option<String> = matches.value_of("title").map(String::from);
  let tags: Option<BTreeSet<String>> = matches
        .values_of("tags")
        .map(|tags| tags.map(String::from).collect());

  let new_note = note::Note { id: old_note.id
                            , title: title.unwrap_or(old_note.title)
                            , tags: tags.unwrap_or(old_note.tags)
                            , body: body.unwrap_or(old_note.body)
                            };

  cache.insert(new_note.id, new_note);
  
  // save the updated cache
  try!(write_to_default_cache(&cache));
  println!("Note cache updated.");

  Ok(())
}


pub fn export_notes(matches: &ArgMatches) -> Result<(),Error> {

  let patharg = matches.value_of("path").unwrap();
  let relative = matches.is_present("relative");

  // load the matching notes from the cache
  let cache = try!(load_from_default_cache());

  let mut file = try!(File::create(patharg).map_err(|_| Error::CreateExport));

  let matching = cache
    .values()
    .filter(|note| -> bool 
      { matches.value_of("title").map_or(true, |title| note.filter_title(title))
      & matches.values_of("body").map_or(true, |mut bodies| bodies.any(|body| note.filter_body(body)))
      & matches.values_of("id").map_or(true, |mut ids| ids.any(|id| note.filter_id(id)))
      & matches.values_of("tags").map_or(true, |tags| note.filter_tags(tags.map(String::from).collect())) });

  // branch depending on if we want our export to be relative or not
  let to_save: Vec<note::Note> = try!(
    if relative {
      // For relative, we want to compare the given path to the ones of notes, so we need
      // both of these to be canonicalized.
      let path = try!(canonicalize(patharg).map_err(|_| Error::Canonicalize(String::from(patharg))));
      
      let mut folder = path.clone();
      folder.pop();

      matching.map(|note| {
        let body_path = PathBuf::from(note.body.clone());

        path_relative_from(&body_path, &folder)
          .and_then(|buf| buf.to_str().map(|b| String::from(b)))
          .map(|new_body|
            note::Note { id: note.id.clone()
                       , title: note.title.clone()
                       , tags: note.tags.clone()
                       , body: String::from(new_body)
                       }
            )
          .ok_or(Error::RelativePath(note.id))
      }).collect()
    } else {
      // otherwise, we just want to save `matching` straight up as is.
      Ok(matching.map(|note| note.clone()).collect())
    }
  );

  let encoded = try!(json::encode(&to_save).map_err(|_| Error::JsonEncode));

  try!(file.write(encoded.as_bytes()).map_err(|_| Error::WriteExport));

  Ok(())
}


pub fn import_notes(matches: &ArgMatches) -> Result<(),Error> {

  fn prompt_user(note: &note::Note) -> bool {
    use std::io::{stdin,stdout,Write};

    println!("A note with ID {:016X}] already exists. Do you want to overwrite it?", note.id);
    let _ = stdout().flush();

    loop {
      let mut input = String::new();
      let _ = stdin().read_line(&mut input);
      
      if input.starts_with("y") || input.starts_with("yes") {
        return true;
      } else if input.starts_with("n") || input.starts_with("no") {
        return false;
      } else {
        println!("Invalid response. Expecting 'yes' or 'no'.");
      }
    };
  }

  let patharg = matches.value_of("path").unwrap();
  let relative = matches.is_present("relative");

  // load the matching notes from the cache
  let mut cache = load_from_default_cache().ok().unwrap_or_default();

  let mut file = try!(File::open(patharg).map_err(|_| Error::ReadExport));
  let mut data = String::new();
  try!(file.read_to_string(&mut data).map_err(|_| Error::ReadExport));

  let to_import: Vec<note::Note> = try!(json::decode(&data).map_err(|_| Error::JsonDecode));

  // branch depending on if we want our import to be relative or not
  for note in to_import {
    let new_body = if relative {
      let mut path1 = PathBuf::new();
      path1.push(patharg);
      path1.pop();
      path1.push(note.body.clone());
      
      let path = try!(canonicalize(path1).map_err(|_| Error::Canonicalize(String::from(patharg))));

      String::from(try!(path.to_str().ok_or(Error::Canonicalize(String::from(patharg)))))
    } else {
      note.body.clone()
    };

    if !cache.contains_key(&note.id) || prompt_user(&note) {
      let _ = cache.insert(note.id, note::Note { id: note.id, title: note.title, tags: note.tags, body: new_body });
    }
  }

  // save the updated cache
  try!(write_to_default_cache(&cache));
  println!("Note cache updated.");
    
  Ok(())
}


// This routine is adapted from the *old* Path's `path_relative_from`
// function, which works differently from the new `relative_from` function.
// In particular, this handles the case on unix where both paths are
// absolute but with only the root as the common directory.
fn path_relative_from(path: &PathBuf, base: &PathBuf) -> Option<PathBuf> {
    use std::path::Component;

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => return None,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}
