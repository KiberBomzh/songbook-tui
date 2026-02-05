pub mod txt_reader;

use std::path::Path;
use std::fs;
use std::io::Result;
use crate::{Song, Metadata};


impl Song {
    pub fn from_str(song: &str, meta: &str) -> anyhow::Result<Self> {
        let (blocks, chord_list) = txt_reader::read_from_txt(song);
        return Ok( Self { blocks, chord_list, metadata: Metadata::from_str(meta)? } )
    }

    pub fn from_txt(file_path: &Path, metadata: Metadata) -> Result<Self> {
        let (blocks, chord_list) = txt_reader::read_from_txt(
            &fs::read_to_string(file_path)?
        );

        // Написать определение тональности
        Ok( Self { blocks, chord_list, metadata} )
    }

    // pub fn from_chordpro(file_path: &Path) -> Result<Song> { }
}
