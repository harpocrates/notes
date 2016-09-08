extern crate rustc_serialize;
extern crate rand;
extern crate bincode;
extern crate regex;
extern crate open;
extern crate clap;

use clap::{Arg,App,SubCommand};

mod note;
mod cli;

fn main() {

  let filter_args = [
    Arg::with_name("title").long("title").short("t").help("regex to match to title of the note").takes_value(true),
    Arg::with_name("body").long("body").short("b").help("note with given body path").takes_value(true).multiple(true),
    Arg::with_name("id").long("id").short("i").help("note with exactly the given id").takes_value(true).multiple(true),
    Arg::with_name("tags").long("tags").short("a").help("note having all given tags").takes_value(true).multiple(true),
    Arg::with_name("lines").long("lines").short("n").help("limit for number of notes, defaults to 10").takes_value(true),
  ];

  let app_m = App::new("notes")
    .subcommand(
        SubCommand::with_name("new")
          .about("Used for creating new notes")
          .arg(Arg::with_name("title").long("title").short("t").help("title of the note").takes_value(true).required(true))
          .arg(Arg::with_name("tags").long("tags").short("a").help("tags for the note").takes_value(true).multiple(true))
          .arg(Arg::with_name("body").long("body").short("b").help("path to body of note").takes_value(true).required(true))
      )
    .subcommand(
        SubCommand::with_name("open")
          .about("Used for opening notes")
          .args(&filter_args)
      )
    .subcommand(
        SubCommand::with_name("list")
          .about("Used for listing notes")
          .args(&filter_args)
      )
    .subcommand(
        SubCommand::with_name("update")
          .about("Used for updating the tags/title/body of a note")
          .arg(Arg::with_name("id").long("id").short("i").help("id of the note to update").takes_value(true))
          .arg(Arg::with_name("title").long("title").short("t").help("title of the note").takes_value(true))
          .arg(Arg::with_name("tags").long("tags").short("a").help("tags for the note").takes_value(true).multiple(true))
          .arg(Arg::with_name("body").long("body").short("b").help("path to body of note").takes_value(true))
      )
    .subcommand(
      SubCommand::with_name("drop")
          .about("Used for forgetting notes")
          .args(&filter_args)
          .arg(Arg::with_name("force").long("force").short("f").help("don't ask user before dropping each note"))
      )
    .subcommand(
      SubCommand::with_name("export")
          .about("Used for exporting notes to JSON")
          .args(&filter_args)
          .arg(Arg::with_name("path").long("path").short("p").help("where to export notes").takes_value(true).required(true))
          .arg(Arg::with_name("relative").long("relative").short("r").help("export with file-paths relative to export location"))
      )
    .subcommand(
      SubCommand::with_name("import")
          .about("Used for importing notes to the note cache")
          .arg(Arg::with_name("path").long("path").short("p").help("from where to import notes").takes_value(true).required(true))
          .arg(Arg::with_name("relative").long("relative").short("r").help("export with file-paths relative to export location"))
          .arg(Arg::with_name("force").long("force").short("f").help("don't ask user before overwriting notes"))
      )
    .get_matches();

  let result = match app_m.subcommand() {
    ("new",    Some(new_m))    => cli::new_note(new_m),
    ("open",   Some(open_m))   => cli::open_list_notes(open_m, true),
    ("list",   Some(list_m))   => cli::open_list_notes(list_m, false),
    ("update", Some(update_m)) => cli::update_note(update_m),
    ("drop",   Some(drop_m))   => cli::drop_notes(drop_m),
    ("export", Some(export_m)) => cli::export_notes(export_m), 
    ("import", Some(import_m)) => Err(String::from("Unimplemented")), 
     _                         => Err(String::from("Unimplemented")),
  };

  for e in result.err() {
    println!("Notes ran into a problem: {}", e);
  }
}


/*
TODO:

 - convert error handling to Result<(),...some enum type...>
 - add support for tracking files when they move (inotify)
 - add export option to just export json, or to collect json + files
*/
