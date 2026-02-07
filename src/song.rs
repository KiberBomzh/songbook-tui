pub mod block;
pub mod row;
pub mod chord;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use crate::Fingering;
use crate::{BLOCK_START, BLOCK_END, STANDART_TUNING};
use crate::Note;
use crate::sum_text_in_fingerings;
use crate::song::chord::Chord;
use crate::song::block::{Block, Line};


#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    pub metadata: Metadata,
    pub chord_list: Vec<Chord>,
    pub blocks: Vec<Block>,
}
// Тональности:
// Am - C
// A#m - C#
// Bm - D
// Cm - D#
// C#m - E
// Dm - F
// D#m - F#
// Em - G
// Fm - G#
// F#m - A
// Gm - A#
// G#m - B

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub key: Option<Note>
}

impl Metadata {
    pub fn from_str(meta: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(meta)?)
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(&self)?)
    }
}


impl Song {
    pub fn get_song_as_text(&self, chords: bool, rhythm: bool) -> String {
        let mut s = String::new();


        if !self.metadata.artist.is_empty() && !self.metadata.title.is_empty() {
            s.push_str( &format!("{} - {}", self.metadata.artist, self.metadata.title) );
            s.push_str("\n\n");
        }

        if chords {
            let mut fings = Vec::new();
            for f in self.get_fingerings() {
                fings.push(f[0].clone());
            }
            if let Some(text) = sum_text_in_fingerings(&fings) {
                s.push_str(&text);
            }
        }

        s.push_str(&self.to_string(chords, rhythm));


        return s
    }

    pub fn print(&self, chords: bool, rhythm: bool) {
        println!("{}", self.get_song_as_text(chords, rhythm));
    }

    pub fn to_string(&self, chords: bool, rhythm: bool) -> String {
        let mut s = String::new();
        let mut is_first = true;
        for block in &self.blocks {
            if is_first { is_first = false }
            else { s.push_str("\n\n") }

            if let Some(title) = &block.title {
                s.push_str(&title);
                if !block.lines.is_empty() { s.push('\n') }
            }
            let mut is_first_line = true;
            for line in &block.lines {
                if is_first_line { is_first_line = false }
                else { s.push('\n') }
                match line {
                    Line::TextBlock(row) => s.push_str(&row.to_string(chords, rhythm)),
                    Line::ChordsLine(chords) => {
                        for chord in chords {
                            s.push_str(&chord.text);
                            s.push(' ');
                        }
                    },
                    Line::EmptyLine => {}
                }
            }
        }

        return s
    }

    pub fn transpose(&mut self, steps: i32) {
        if let Some(key) = self.metadata.key {
            self.metadata.key = Some(key.transpose(steps))
        }
        for chord in &mut self.chord_list { *chord = chord.transpose(steps) }
        for block in &mut self.blocks {
            for line in &mut block.lines {
                match line {
                    Line::TextBlock(row) => {
                        if let Some(chords) = &mut row.chords {
                            for (key, value) in chords.clone() {
                                chords.insert(key, value.transpose(steps));
                            }
                        }
                    },
                    Line::ChordsLine(chords) =>
                        chords.iter_mut().for_each(|c| *c = c.transpose(steps)),
                    Line::EmptyLine => {}
                }
            }
        }
    }

    pub fn get_fingerings(&self) -> Vec<Vec<Fingering>> {
        let mut fings = Vec::new();
        for chord in &self.chord_list {
            fings.push(chord.get_fingerings(&STANDART_TUNING));
        }

        return fings
    }

    pub fn get_for_editing(&self) -> String {
        let mut s = String::new();
        let mut is_first = true;
        for block in &self.blocks {
            if is_first { is_first = false }
            else { s.push_str("\n\n") }
            block.get_for_editing(&mut s);
        }

        return s
    }

    pub fn change_from_edited_str(&mut self, text: &str) {
        let mut blocks: Vec<Block> = Vec::new();

        let mut block_buf = String::new();
        let mut is_in_block = false;
        for line in text.lines() {
            if line.starts_with(BLOCK_START) { is_in_block = true }
            else if line.starts_with(BLOCK_END) {
                is_in_block = false;
                blocks.push( Block::from_edited(&block_buf) );
                block_buf.clear();
            } else if is_in_block { block_buf.push_str(line); block_buf.push('\n'); }
        }


        self.blocks = blocks;
        self.chord_list = self.get_chord_list();
    }

    fn get_chord_list(&self) -> Vec<Chord> {
        let mut list = Vec::new();
        for block in &self.blocks {
            for line in &block.lines {
                match line {
                    Line::TextBlock(row) => {
                        if let Some(chords) = &row.chords {
                            for (_, chord) in chords {
                                if list.iter().all(|c| c != chord) {
                                    list.push(chord.clone());
                                }
                            }
                        }
                    },
                    Line::ChordsLine(chords) => for chord in chords {
                        if list.iter().all(|c| c != chord) {
                            list.push(chord.clone());
                        }
                    },
                    Line::EmptyLine => {}
                }
            }
        }

        return list;
    }
}
