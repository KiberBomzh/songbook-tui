mod txt_reader;

use std::path::Path;
use std::io::Result;
use crate::Song;


impl Song {
    pub fn from_txt(file_path: &Path, metadata: crate::Metadata) -> Result<Song> {

        Ok( Song {
            blocks: txt_reader::read_from_txt(file_path)?,
            metadata
        } )
    }

    // pub fn from_chordpro(file_path: &Path) -> Result<Song> { }
}
