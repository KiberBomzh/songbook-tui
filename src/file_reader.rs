mod txt_reader;

use std::path::Path;
use std::io::Result;
use crate::Song;


impl Song {
    pub fn from_txt(file_path: &Path, metadata: crate::Metadata) -> Result<Song> {
        let (blocks, chord_list) = txt_reader::read_from_txt(file_path)?;

        // Написать определение тональности
        Ok( Song { blocks, chord_list, metadata, key: crate::Note::C } )
    }

    // pub fn from_chordpro(file_path: &Path) -> Result<Song> { }
}
