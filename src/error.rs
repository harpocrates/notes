pub enum Error {
  OpenCache,
  DecodeCache,
  CreateCache,
  EncodeCache,
  HomeDir,
  MalformedId,
  NoSuchNote(usize),
  CreateExport,
  WriteExport,
  ReadExport,
  Canonicalize(String),
  RelativePath(usize),
  JsonEncode,
  JsonDecode,
}

pub fn print_error(e: Error) -> String {
  match e {
    Error::OpenCache => String::from("failed to open the notes cache"),
    Error::DecodeCache => String::from("failed to decode the notes cache"),
    Error::CreateCache => String::from("failed to create the notes cache"),
    Error::EncodeCache => String::from("failed to encode the notes cache"),
    Error::HomeDir => String::from("failed to find your home directory"),
    Error::MalformedId => String::from("id could not be parsed"),
    Error::NoSuchNote(id) => format!("no note with id '{:016X}' found", id),
    Error::CreateExport => String::from("failed to create export file"),
    Error::WriteExport => String::from("failed to write export file"),
    Error::ReadExport => String::from("failed to read import file"),
    Error::Canonicalize(path) => format!("failed to canonicalize path '{}'", path),
    Error::RelativePath(id) => format!("failed to get relative path of note '{:016X}'", id),
    Error::JsonEncode => String::from("failed to encode json"),
    Error::JsonDecode => String::from("failed to decode json"),
  }
}
