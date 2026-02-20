pub mod txt_reader;
pub mod chordpro_reader;

use std::path::Path;
use std::fs;
use std::io::Result;
use crate::{Song, Metadata};


impl Song {
    pub fn from_str(song: &str, meta: &str) -> anyhow::Result<Self> {
        let (blocks, chord_list) = txt_reader::read_from_txt(song);
        return Ok( Self { blocks, chord_list, metadata: Metadata::from_str(meta)?, notes: None } )
    }

    pub fn from_txt(file_path: &Path, title: &str, artist: &str) -> Result<Self> {
        let (blocks, chord_list) = txt_reader::read_from_txt(
            &fs::read_to_string(file_path)?
        );
        let metadata = Metadata {
            artist: artist.to_string(),
            title: title.to_string(),
            key: None,
            capo: None,
            autoscroll_speed: None
        };
        let mut song = Self { blocks, chord_list, metadata, notes: None };
        song.detect_key();


        Ok(song)
    }

    pub fn from_chordpro(file_path: &Path) -> Result<Self> {
        let (metadata, blocks, chord_list) = chordpro_reader::read_from_chordpro(
            &fs::read_to_string(file_path)?
        );
        let metadata = metadata.expect("Cannot read metadata(title or artist)!");
        Ok( Self { blocks, chord_list, metadata, notes: None } )
    }
}
