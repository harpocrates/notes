use bincode::rustc_serialize::{encode_into, decode_from};
use rustc_serialize::json;
use std::fs::{File, canonicalize};
use std::io::Write;
use std::collections::{BTreeSet, BTreeMap};
use std::env;
use bincode::SizeLimit;
use std::path::{Path, PathBuf};
use rand;
use clap::ArgMatches;

use note;


// Load notes from cache
pub fn load_from_cache(location: &Path) -> Result<BTreeMap<usize,note::Note>,String> {
  let mut file = try!(File::open(location).map_err(|_| "cannot open cache"));
  decode_from(&mut file, SizeLimit::Infinite)
    .map_err(|_| String::from("cannot decode cache"))
}

// Write notes to cache
pub fn write_to_cache(existing: &BTreeMap<usize,note::Note>, location: &Path) -> Result<(),String> {
  let mut file = try!(File::create(location).map_err(|_| "cannot create cache"));
  encode_into(existing, &mut file, SizeLimit::Infinite)
    .map_err(|_| String::from("cannot encode cache"))
}

// Load notes from cache
pub fn load_from_default_cache() -> Result<BTreeMap<usize,note::Note>,String> {
  let mut home = try!(env::home_dir().ok_or("failed to find home directory"));
  home.push(".notes-cache");
  load_from_cache(home.as_path())
}

// Write notes to cache
pub fn write_to_default_cache(existing: &BTreeMap<usize,note::Note>) -> Result<(),String> {
  let mut home = try!(env::home_dir().ok_or("failed to find home directory"));
  home.push(".notes-cache");
  write_to_cache(existing, home.as_path())
}


pub fn open_list_notes(matches: &ArgMatches, open: bool) -> Result<(),String> {

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
  
  // print the matching notes
  println!("Note UUID        | Title                | Tags                    | Date     ");
  println!("-----------------+----------------------+-------------------------+----------");

  for note in matching.by_ref().take(num_display) {
    println!("{:016X} | {: <20} | {: <23} | ",
      note.id,
      note.title,
      note.tags.iter().fold(String::new(), |s,tag| if s!="" { s+" "+ &tag } else { s+&tag }),
    );

    if open {
      note.open();
    }
  }

  // remind the user about remaining undisplayed/unopened notes (if there are any)
  let remaining = matching.count();
  if remaining > 0 {
    println!("There are {} matching notes not {}.", remaining, if open { "opened" } else { "listed" });
  }

  Ok(())
}


pub fn drop_notes(matches: &ArgMatches) -> Result<(),String> {
  
  fn prompt_user(note: &note::Note) -> bool {
    use std::io::{stdin,stdout,Write};

    println!("Are you sure you want to delete note '{}' [{}]", note.title, note.id.to_string());
    let _ = stdout().flush();

    loop {
      let mut input = String::new();
      let _ = stdin().read_line(&mut input);
      match input.as_ref() {
        "y" | "yes" => return true,
        "n" | "no"  => return false,
        _ => println!("Invalid response. Expecting 'yes' or 'no'."),
      }
    };
  }

  // load the matching notes from the cache
  let old_cache = try!(load_from_default_cache());

  let new_cache = old_cache
    .into_iter()
    .filter(|&(_, ref note)| -> bool {
        (!matches.value_of("title").map_or(true, |title| note.filter_title(title))
        | !matches.values_of("body").map_or(true, |mut bodies| bodies.any(|body| note.filter_body(body)))
        | !matches.values_of("id").map_or(true, |mut ids| ids.any(|id| note.filter_id(id)))
        | !matches.values_of("tags").map_or(true, |tags| note.filter_tags(tags.map(String::from).collect())))
      && matches.is_present("force") || prompt_user(&note) 
    })
    .collect();

  // save the updated cache
  try!(write_to_default_cache(&new_cache));
  println!("Note updated.");

  Ok(())
}



pub fn new_note(matches: &ArgMatches) -> Result<(),String> {

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


pub fn update_note(matches: &ArgMatches) -> Result<(),String> {

  // load the cache
  let mut cache = load_from_default_cache().ok().unwrap_or_default();

  // find and remove from the cache the note
  let old_note = try!(usize::from_str_radix(matches.value_of("id").unwrap(),16)
    .map_err(|_| "malformed id given")
    .and_then(|id| cache.remove(&id).ok_or("No note with specified id found")));

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
  println!("Note updated.");

  Ok(())
}

// TODO relative
pub fn export_notes(matches: &ArgMatches) -> Result<(),String> {

  let patharg = matches.value_of("path").unwrap();
  let relative = matches.is_present("relative");

  // load the matching notes from the cache
  let cache = try!(load_from_default_cache());

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
      let path = try!(canonicalize(patharg)
        .map_err(|_| format!("Could not canonicalize path given '{}'.", patharg)));

      matching.map(|note| {
        let body_path = PathBuf::from(note.body.clone());

        path_relative_from(&body_path, &path)
          .and_then(|buf| buf.to_str().map(|b| String::from(b)))
          .map(|new_body|
            note::Note { id: note.id.clone()
                       , title: note.title.clone()
                       , tags: note.tags.clone()
                       , body: String::from(new_body)
                       }
            )
          .ok_or(format!("Failed to get relative path of note '{}'.", note.id))
      }).collect()
    } else {
      // otherwise, we just want to save `matching` straight up as is.
      Ok(matching.map(|note| note.clone()).collect())
    }
  );

  let encoded = json::encode(&to_save).unwrap();

  File::create(patharg)
    .ok()
    .and_then(|mut file| file.write(encoded.as_bytes()).ok());
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