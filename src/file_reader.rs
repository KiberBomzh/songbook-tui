pub mod txt_reader;

use std::path::Path;
use std::fs;
use std::io::Result;
use crate::Song;


impl Song {
    pub fn from_txt(file_path: &Path, metadata: crate::Metadata) -> Result<Song> {
        let (blocks, chord_list) = txt_reader::read_from_txt(
            &fs::read_to_string(file_path)?
        );

        // Написать определение тональности
        Ok( Song { blocks, chord_list, metadata} )
    }

    // pub fn from_chordpro(file_path: &Path) -> Result<Song> { }
}
