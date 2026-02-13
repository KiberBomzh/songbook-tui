pub mod txt_reader;
pub mod chordpro_reader;

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
        let mut song = Self { blocks, chord_list, metadata };
        if song.metadata.key == None {
            song.detect_key();
        }


        Ok(song)
    }

    pub fn from_chordpro(file_path: &Path) -> Result<Self> {
        let (metadata, blocks, chord_list) = chordpro_reader::read_from_chordpro(
            &fs::read_to_string(file_path)?
        );
        let metadata = metadata.expect("Cannot read metadata(title or artist)!");
        Ok( Self { blocks, chord_list, metadata} )
    }
}
