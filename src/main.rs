extern crate getopts;
extern crate rustc_serialize;
extern crate rand;
extern crate bincode;
extern crate regex;
extern crate open;

use std::env;

mod note;
mod cli;
// mod note;
// mod cli;

fn main() {

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    println!("Not enough args."); // TODO print avaiable options
    return
  }


  match args[1].as_str() {
    "new"  => cli::new_note(args),
    "open" => cli::open_list_notes(args, false),
    "list" => cli::open_list_notes(args, true),
    // "config" => ,
    // "update" => ,
    "help" | _ => println!("Unimplemented") // TODO print avaiable options
  };

}
