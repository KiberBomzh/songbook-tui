use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{BufWriter, BufReader, Write, Error, ErrorKind, stdout};
use std::process::{Command, Stdio};

use dirs;
use anyhow::Result;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor}
};

use crate::{Song, Metadata};
use crate::file_reader::txt_reader::read_from_txt;


const FORBIDDEN_CHARS: [char; 9] = ['<', '>', ':', '/', '\\', '|', '?', '*', '`'];



pub fn show(song_path: &Path, key: Option<crate::Note>) -> Result<()> {
    let mut path = get_lib_path()?;
    path = path.join(song_path);

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut song: Song = serde_yaml::from_reader(reader)?;
    if let Some(k) = key {
        if let Some(mut m_key) = song.metadata.key {
            while m_key != k {
                song.transpose(1);
                m_key = song.metadata.key.unwrap();
            }
        } else { println!("Add a key before transposing, try 'songbook edit <song_name> -t meta'") }
    }

    let text = song.get_song_as_text();
    print(&text)?;


    Ok(())
}


pub fn edit(added_path: &Path, target: &str) -> Result<()> {
    let mut path = get_lib_path()?;
    path = path.join(added_path);
    if !path.exists() {
        return Err( Error::new(ErrorKind::NotFound, "There's no such file!").into() )
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut song: Song = serde_yaml::from_reader(reader)?;

    match target {
        "meta" => {
            let metadata = &mut song.metadata;
            let mut data = Metadata::to_string(&metadata)?;
            data = edit::edit(data)?;

            *metadata = Metadata::from_str(&data)?;
        },
        "chords" => {
            let mut text = song.to_string();
            text = edit::edit(text)?;

            let blocks = &mut song.blocks;
            let chord_list = &mut song.chord_list;
            (*blocks, *chord_list) = read_from_txt(&text);
        },
        "rhythm" => {
            let mut text = song.get_rhythm_for_editing();
            text = edit::edit(text)?;
            
            song.change_rhythm_from_edited_str(&text);
        },
        _ => { println!("There's no such option!"); return Ok(()) }
    }

    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, &song)?;


    Ok(())
}


pub fn add(song: &Song) -> Result<()> {
    let mut path = get_lib_path()?;
    if !path.exists() { fs::create_dir_all(&path)? }

    let song_name = get_without_forbidden_chars(
        format!("{} - {}", song.metadata.artist, song.metadata.title)
    );
    path.push(&song_name);
    path = get_free_path(path, &song_name);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, &song)?;


    Ok(())
}


pub fn rm(added_path: &Path) -> Result<()> {
    let mut path = get_lib_path()?;
    path = path.join(added_path);

    if path.exists() {
        if path.is_file() { fs::remove_file(path)? }
        else if path.is_dir() { fs::remove_dir_all(path)? }
    } else {
        return Err( Error::new(
            ErrorKind::NotFound,
            format!("There's no such path: {:#?}", added_path)
        ).into())
    }

    Ok(())
}


pub fn mv(input_path: &Path, output_path: &Path) -> Result<()> {
    let path = get_lib_path()?;
    let i_path = path.join(input_path);
    if !i_path.exists() {
        return Err( Error::new(
            ErrorKind::NotFound,
            format!("There's no such path: {:#?}", input_path)
        ).into())
    }

    let mut o_path = path.join(output_path);
    if o_path.is_dir() { o_path = o_path.join( i_path.file_name()
        .expect("Cannot get input_path file name!") ) }

    fs::rename(i_path, o_path)?;

    Ok(())
}


pub fn ls(added_path: Option<&Path>) -> Result<()> {
    let mut path = get_lib_path()?;
    if let Some(p) = added_path { path = path.join(p) }
    if !path.exists() {
        return Err( Error::new(ErrorKind::NotFound, "There's no such dir!").into() )
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if entry.path().is_dir() {
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Blue),
                    Print(name),
                    Print("\n"),
                    ResetColor
                )?;
            } else {
                println!("{}", name);
            }
        }
    }

    Ok(())
}


pub fn tree(added_path: Option<&Path>) -> Result<()> {
    let mut path = get_lib_path()?;
    if let Some(p) = added_path { path = path.join(p) }
    if !path.is_dir() {
        return Err( Error::new(ErrorKind::NotFound, "There's no such dir!").into() )
    }

    recursive_tree(&path, 1)?;

    Ok(())
}

fn recursive_tree(dir: &Path, indent: usize) -> Result<()> {
    if let Some(name) = dir.file_name().and_then(|f| f.to_str()) {
        println!("{}", name);
    }

    let last = if let Some(entry) = fs::read_dir(dir)?.last() { entry?.path() }
    else { return Ok(()) };

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if indent > 1 {
                print!("{}", "│   ".repeat(indent - 1));
            }

            if entry.path() == last {
                print!("└── ");
            } else {
                print!("├── ");
            }

            if entry.path().is_dir() {
                recursive_tree(&entry.path(), indent + 1)?;
            } else {
                println!("{}", name);
            }
        }
    }

    Ok(())
}


pub fn mkdir(added_path: &Path) -> Result<()> {
    let mut path = get_lib_path()?;
    path = path.join(added_path);

    if path.exists() {
         
        return Err( Error::new(
            ErrorKind::AlreadyExists, 
            format!("{:#?} is already exists!", added_path)
        ).into())
    }

    fs::create_dir_all(path)?;

    Ok(())
}


fn print(text: &str) -> Result<()> {
    if let Ok(mut child) = Command::new("less").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
    } else {
        println!("{text}");
    }

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

fn get_lib_path() -> Result<PathBuf> {
    if let Some(mut path) = dirs::data_dir() {
        path.push("songbook");
        path.push("library");

        Ok(path)
    }
    else { Err( Error::new(ErrorKind::NotFound, "Cannot get data directory!").into() ) }
}
