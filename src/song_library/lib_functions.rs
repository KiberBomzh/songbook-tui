use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{Error, ErrorKind, BufReader};

use anyhow::Result;

use crate::Song;
use crate::song_library::get_lib_path;


// returns files in directory(file_name, file_path) and directory
pub fn get_files_in_dir(added_path: Option<&Path>) -> Result<(Vec<(String, PathBuf)>, PathBuf)> {
    let mut path = get_lib_path()?;
    if let Some(p) = added_path { path = path.join(p) }
    if !path.exists() {
        return Err( Error::new(ErrorKind::NotFound, "There's no such dir!").into() )
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            let name = if entry.path().is_dir() { format!("📁{}", name) }
            else { format!("{}", name) };
            files.push( (name, entry.path()) );
        }
    }

    Ok( (files, path) )
}


pub fn get_song(song_path: &Path) -> Result<Song> {
    let mut path = get_lib_path()?;
    path = path.join(song_path);

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let song: Song = serde_yaml::from_reader(reader)?;

    Ok(song)
}
