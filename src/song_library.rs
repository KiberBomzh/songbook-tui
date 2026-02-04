use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{BufWriter, Error};
use std::io::ErrorKind;

use dirs;


const FORBIDDEN_CHARS: [char; 9] = ['<', '>', ':', '/', '\\', '|', '?', '*', '`'];



pub fn add(song: &crate::Song) -> Result<(), Error> {
    let mut path = get_lib_path()?;
    if !path.exists() { fs::create_dir_all(&path)? }

    let song_name = get_without_forbidden_chars(
        format!("{} - {}", song.metadata.artist, song.metadata.title)
    );
    path.push(&song_name);
    path = get_free_path(path, &song_name);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &song)?;


    Ok(())
}


pub fn rm(added_path: &Path) -> Result<(), Error> {
    let mut path = get_lib_path()?;
    path = path.join(added_path);

    if path.exists() {
        if path.is_file() { fs::remove_file(path)? }
        else if path.is_dir() { fs::remove_dir_all(path)? }
    } else {
        return Err( Error::new(
            ErrorKind::NotFound,
            format!("There's no such path: {:#?}", added_path)
        ))
    }

    Ok(())
}


pub fn mv(input_path: &Path, output_path: &Path) -> Result<(), Error> {
    let path = get_lib_path()?;
    let i_path = path.join(input_path);
    if !i_path.exists() {
        return Err( Error::new(
            ErrorKind::NotFound,
            format!("There's no such path: {:#?}", input_path)
        ))
    }

    let o_path = path.join(output_path);

    fs::rename(i_path, o_path)?;

    Ok(())
}


pub fn ls(added_path: Option<&Path>) -> Result<(), Error> {
    let mut path = get_lib_path()?;
    if let Some(p) = added_path { path = path.join(p) }
    if !path.exists() {
        return Err( Error::new(ErrorKind::NotFound, "There's no such dir!") )
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if entry.path().is_dir() {
                println!("{}", name.blue());
            } else {
                println!("{}", name);
            }
        }
    }

    Ok(())
}


pub fn mkdir(added_path: &Path) -> Result<(), Error> {
    let mut path = get_lib_path()?;
    path = path.join(added_path);

    if path.exists() {
         
        return Err( Error::new(
            ErrorKind::AlreadyExists, 
            format!("{:#?} is already exists!", added_path)
        ))
    }

    fs::create_dir_all(path)?;

    Ok(())
}


fn get_without_forbidden_chars(text: String) -> String {
    text.chars().map(|c|
        if FORBIDDEN_CHARS.iter().any(|f| *f == c) { '_' }
        else { c }
    ).collect()
}

fn get_free_path(mut path: PathBuf, name: &str) -> PathBuf {
    let mut counter = 1;
    while path.exists() {
        path.set_file_name(&format!("{}({})", name, counter));
        counter += 1;
    }

    return path
}

fn get_lib_path() -> Result<PathBuf, Error> {
    if let Some(mut path) = dirs::data_dir() {
        path.push("songbook");
        path.push("library");

        Ok(path)
    }
    else { Err( Error::new(ErrorKind::NotFound, "Cannot get data directory!") ) }
}
