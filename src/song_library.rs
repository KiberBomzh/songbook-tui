use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{BufWriter, BufReader, Write, Error, ErrorKind, stdout};
use std::process::{Command, Stdio};

use dirs;
use include_dir::{include_dir, Dir};
use anyhow::Result;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor}
};

use crate::{Song, Metadata, Fingering};


const FORBIDDEN_CHARS: [char; 9] = ['<', '>', ':', '/', '\\', '|', '?', '*', '`'];



pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let assets: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");
    let mut path: PathBuf = dirs::data_dir()
        .ok_or("Cannot get path for data!")?;
    path.push("songbook");
    if !path.exists() { fs::create_dir_all(&path)? }
    assets.extract(&path)?;

    Ok(())
}

pub fn show(
    song_path: &Path,
    key: Option<crate::Note>,
    chords: bool, // show chords
    rhythm: bool, // show rhythm
    fingerings: bool, // show fingerings
    is_colored: bool
) -> Result<()> {
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

    if is_colored {
        if let Ok(mut child) = Command::new("less")
                                .arg("-R")
                                .stdin(Stdio::piped())
                                .spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                song.print_colored(&mut stdin, chords, rhythm, fingerings)?
            }
            child.wait()?;
        } else {
            song.print_colored(&mut std::io::stdout(), chords, rhythm, fingerings)?;
        }
    } else {
        let text = song.get_song_as_text(chords, rhythm, fingerings);
        print(&text)?;
    }


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
        "song" => {
            let mut text = song.get_for_editing();
            text = edit::edit(text)?;
            
            song.change_from_edited_str(&text);
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


pub fn sort() -> Result<()> {
    let path = get_lib_path()?;
    recursive_sort(&path, &path)?;
    loop {
        if !remove_empty_folders(&path)? { break }
    }

    Ok(())
}
fn recursive_sort(path: &Path, lib_path: &Path) -> Result<()> {
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        if entry.path().is_dir() { recursive_sort(&entry.path(), lib_path)?; continue }

        let file = File::open(entry.path())?;
        let reader = BufReader::new(file);
        let song: Song = serde_yaml::from_reader(reader)?;

        let artist = get_without_forbidden_chars(song.metadata.artist.clone());
        let title = get_without_forbidden_chars(song.metadata.title.clone());
        let mut new_path = lib_path.join(&artist);
        if !new_path.exists() { fs::create_dir_all(&new_path)? }
        new_path = new_path.join(&title);
        let is_the_same = if new_path != entry.path() {
            new_path = get_free_path(new_path, &title);
            false
        } else { true };

        let file = File::create(&new_path)?;
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &song)?;

        if !is_the_same {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}
fn remove_empty_folders(path: &Path) -> Result<bool> {
    let mut is_something_deleted = false;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let current_path = entry.path();
        if current_path.is_file() { continue }

        if current_path.is_dir() {
            if fs::read_dir(&current_path)?.next().is_none() {
                fs::remove_dir(&current_path)?;
                is_something_deleted = true;
            } else {
                let is_del = remove_empty_folders(&current_path)?;
                if !is_something_deleted { is_something_deleted = is_del }
            }
        }
    }

    return Ok(is_something_deleted)
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

    recursive_tree(&path, 1, false)?;

    Ok(())
}

fn recursive_tree(dir: &Path, indent: usize, is_parent_last: bool) -> Result<()> {
    if let Some(name) = dir.file_name().and_then(|f| f.to_str()) {
        println!("{}", name);
    }

    let last = if let Some(entry) = fs::read_dir(dir)?.last() { entry?.path() }
    else { return Ok(()) };

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let is_last = entry.path() == last;
        if let Some(name) = entry.file_name().to_str() {
            if indent > 1 {
                if is_parent_last {
                    print!("{}", "    ".repeat(indent - 1));
                } else {
                    print!("{}", "│   ".repeat(indent - 1));
                }
            }

            if is_last {
                print!("└── ");
            } else {
                print!("├── ");
            }

            if entry.path().is_dir() {
                recursive_tree(&entry.path(), indent + 1, is_last)?;
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

pub fn add_fingering(fing: &Fingering) -> Result<(), Box<dyn std::error::Error>> {
    let mut path: PathBuf = dirs::data_dir()
        .ok_or("Cannot get path for data!")?;
    path.push("songbook");
    path.push("fingerings");
    if !path.exists() { fs::create_dir_all(&path)? }

    let fing_name = get_without_forbidden_chars(
        fing.get_title()
            .ok_or("Cannot get the fingering title!")?
    );
    path.push(&fing_name);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, &fing)?;

    
    Ok(())
}

pub fn get_fingering(chord_name: &str) -> Result<Option<Fingering>, Box<dyn std::error::Error>> {
    let mut path: PathBuf = dirs::data_dir()
        .ok_or("Cannot get path for data!")?;
    path.push("songbook");
    path.push("fingerings");
    path.push(&chord_name);
    
    if !path.exists() { return Ok(None) }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let fing: Fingering = serde_yaml::from_reader(reader)?;

    
    Ok(Some(fing))
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
