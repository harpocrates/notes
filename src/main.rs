extern crate notes;

use std::env;

fn main() {

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    println!("Not enough args."); // TODO print avaiable options
    return
  }


  match args[1].as_str() {
    "new" => notes::cli::new_note(args),
    "open" => notes::cli::open_list_notes(args, false),
    "list" => notes::cli::open_list_notes(args, true),
    // "config" => ,
    // "update" => ,
    "help" | _ => println!("Unimplemented") // TODO print avaiable options
  };

}
