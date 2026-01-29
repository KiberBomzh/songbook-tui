mod file_reader;

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};


pub fn run() {
    let mut song = Song::from_txt(
        &std::path::PathBuf::from("/home/kiberbomzh/chords.txt"),
        Metadata { title: String::new(), artist: String::new(), key: String::new() }
        ).unwrap();
    song.transpose(2);
    println!("{}", song.get_text());
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    pub metadata: Metadata,
    blocks: Vec<Block>
}

impl Song {
    fn get_text(&self) -> String {
        let mut s = String::new();
        let mut is_first = true;
        for block in &self.blocks {
            if is_first { is_first = false }
            else { s.push_str("\n\n") }

            if let Some(title) = &block.title {
                s.push_str(&title);
                s.push('\n');
            }
            let mut is_first_row = true;
            for row in &block.rows {
                if is_first_row { is_first_row = false }
                else { s.push('\n') }
                s.push_str(&row.get_text());
            }
        }

        return s
    }

    pub fn transpose(&mut self, steps: i32) {
        for block in &mut self.blocks {
            for row in &mut block.rows {
                if let Some(chords) = &mut row.chords {
                    for (key, value) in chords.clone() {
                        chords.insert(key, value.transpose(steps));
                    }
                }
            }
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub key: String // потом сделать перечислением
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

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    title: Option<String>,
    rows: Vec<Row>,
}


#[derive(Serialize, Deserialize, Debug)]
struct Row {
    chords: Option<BTreeMap<usize, String>>,
    text: Option<String>
}

impl Row {
    // Обработать ошибки (когда аккорды накладываются)
    fn get_text(&self) -> String {
        let mut s = String::new();
        if let Some(chords) = &self.chords {
            let mut chords_str = String::new();
            for k in chords.keys() {
                let i: usize;
                let p = k - 1;
                if chords_str.is_empty() {
                    i = p;
                } else {
                    let s_len = chords_str.chars().count();
                    i = p - s_len;
                }
                chords_str.push_str(&" ".repeat(i));
                chords_str.push_str(chords.get(k).unwrap());
            }
            s.push_str(&chords_str);
        }
        if let Some(text) = &self.text {
            if !s.is_empty() { s.push('\n') }
            s.push_str(&text);
        }

        return s
    }
}

