use crate::song::{
    Song,
    Metadata,
    block::{Block, Line},
    row::{Row, ChordPosition},
    chord::Chord
};
use anyhow::Result;


#[derive(serde::Deserialize, Debug)]
struct SongbookPro {
    songs: Vec<SbpSong>,
}

#[derive(serde::Deserialize, Debug)]
struct SbpSong {
    author: String,
    name: String,
    content: String,
    Capo: u8,
    key: u8,
    KeyShift: u8,
    NotesText: String,
}


pub fn read_from_sbp(file_content: &str) 
    -> Result<Vec<Song>>
{
    let sbp: SongbookPro = serde_json::from_str(file_content)?;
    dbg!(&sbp);
    Ok(Vec::new())
}
