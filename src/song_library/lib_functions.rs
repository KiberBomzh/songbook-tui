use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{Error, ErrorKind, BufReader, BufWriter};

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

    let mut buf_for_sorting = Vec::new();
    let mut files = Vec::new();
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if entry.path().is_dir() {
                let name = format!("📁{}", name);
                files.push( (name, entry.path()) );
            }
            else {
                let name = format!("{}", name);
                buf_for_sorting.push( (name, entry.path()) );
            };
        }
    }

    files.extend(buf_for_sorting);
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


pub fn save(song: &Song, path: &Path) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, song)?;

    Ok(())
}


pub fn edit(song: &mut Song) -> Result<()> {
    let mut text = song.get_for_editing();
    text = edit::edit(text)?;
    song.change_from_edited_str(&text);

    Ok(())
}

pub fn find(query: &str) -> Result<Vec<(String, PathBuf)>> {
    let path = get_lib_path()?;
    let mut files = Vec::new();
    recursive_find(&path, &mut files, query)?;

    return Ok(files)
}
fn recursive_find(dir: &Path, files: &mut Vec<(String, PathBuf)>, query: &str) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            recursive_find(&path, files, query)?;
        } else if let Some(name) = path.file_name().and_then(|n: &std::ffi::OsStr| n.to_str()) {
            if name.contains(query) { files.push( (name.to_string(), path.to_path_buf()) ) }
        }
    }

    Ok(())
}
