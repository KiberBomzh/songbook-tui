mod file_reader;
mod chord_fingerings;
mod chord_generator;

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::Chord::{A, ASharp, B, C, CSharp, D, DSharp, E, F, FSharp, G, GSharp};

pub use crate::chord_fingerings::Fingering;
pub use crate::chord_fingerings::StringState::{self, *};



#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    pub metadata: Metadata,
    blocks: Vec<Block>
}

impl Song {
    pub fn get_text(&self) -> String {
        let mut s = String::new();
        let mut is_first = true;
        for block in &self.blocks {
            if is_first { is_first = false }
            else { s.push_str("\n\n") }

            if let Some(title) = &block.title {
                s.push_str(&title);
                if !block.rows.is_empty() { s.push('\n') }
            }
            let mut is_first_row = true;
            for row in &block.rows {
                if is_first_row { is_first_row = false }
                else { s.push('\n') }
                s.push_str(&row.get_text());
            }
        }
        use chord_generator::Note::*;
        dbg!(chord_generator::get_fretboard(&[E, B, G, D, A, E]));

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
    chords: Option<BTreeMap<usize, Chord>>, // позиция в строке - аккорд
    text: Option<String>
}

impl Row {
    fn get_text(&self) -> String {
        let mut s = String::new();
        let mut text = if let Some(t) = &self.text { t.clone() } else { String::new() };
        let mut added_indent = 0;
        if let Some(chords) = &self.chords {
            let mut chords_str = String::new();
            for k in chords.keys() {
                let i: usize;
                let p = k - 1 + added_indent;
                if chords_str.is_empty() {
                    i = p;
                } else {
                    let s_len = chords_str.chars().count();
                    if s_len >= p {
                        let dif = 1 + (s_len - p);
                        added_indent += dif;
                        i = 1;
                        if !text.is_empty() {
                            if let Some(b_index) = get_bytes_index_from_char_index(&text, p) {
                                match text.chars().nth(p) {
                                    Some(c) if c == ' ' => text.insert_str(b_index, &" ".repeat(dif)),
                                    Some(_) => if let Some(prior_char) = text.chars().nth(p - 1) {
                                        if prior_char == ' ' {
                                            text.insert_str(b_index, &" ".repeat(dif))
                                        } else {
                                            text.insert_str(b_index, &"-".repeat(dif))
                                        }
                                    },
                                    None => text.push_str(&" ".repeat(dif))
                                }
                            }
                        }
                    } else {
                        i = p - s_len;
                    }
                }

                chords_str.push_str(&" ".repeat(i));
                chords_str.push_str(&chords.get(k).unwrap().get_text());
            }
            s.push_str(&chords_str);

        }

        if !text.is_empty() {
            if !s.is_empty() { s.push('\n') }
            s.push_str(&text);
        }


        return s
    }
}

fn get_bytes_index_from_char_index(line: &str, char_index: usize) -> Option<usize> {
    line.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Chord {
    A(String),
    ASharp(String),
    B(String),
    C(String),
    CSharp(String),
    D(String),
    DSharp(String),
    E(String),
    F(String),
    FSharp(String),
    G(String),
    GSharp(String)
}

impl Chord {

    pub fn new(s: &str) -> Option<Self> {
        Some( match s {
            s if s.starts_with("A#") => ASharp(s.to_string()),
            s if s.starts_with('A') =>  A(s.to_string()),
            s if s.starts_with('B') =>  B(s.to_string()),
            s if s.starts_with("C#") => CSharp(s.to_string()),
            s if s.starts_with('C') =>  C(s.to_string()),
            s if s.starts_with("D#") => DSharp(s.to_string()),
            s if s.starts_with('D') =>  D(s.to_string()),
            s if s.starts_with('E') =>  E(s.to_string()),
            s if s.starts_with("F#") => FSharp(s.to_string()),
            s if s.starts_with('F') =>  F(s.to_string()),
            s if s.starts_with("G#") => GSharp(s.to_string()),
            s if s.starts_with('G') =>  G(s.to_string()),
            _ => return None
        } )
    }

    pub fn get_text(&self) -> String {
        match self {
            A(text) => text.clone(),
            ASharp(text) => text.clone(),
            B(text) => text.clone(),
            C(text) => text.clone(),
            CSharp(text) => text.clone(),
            D(text) => text.clone(),
            DSharp(text) => text.clone(),
            E(text) => text.clone(),
            F(text) => text.clone(),
            FSharp(text) => text.clone(),
            G(text) => text.clone(),
            GSharp(text) => text.clone()
        }
    }

    pub fn transpose(&self, steps: i32) -> Self {
        let steps = steps % 12;
        if steps == 0 { return self.clone() }
        let mut chord = self.clone();

        if steps > 0 {
            for _ in 0..steps { chord.do_step_right() }
        } else if steps < 0 {
            for _ in steps..0 { chord.do_step_left() }
        }

        return chord
    }

    fn do_step_right(&mut self) {
        *self = match self {
            A(text) =>      ASharp( format!("A#{}", &text[1..]) ),
            ASharp(text) => B( format!("B{}", &text[2..]) ),
            B(text) =>      C( format!("C{}", &text[1..]) ),
            C(text) =>      CSharp( format!("C#{}", &text[1..]) ),
            CSharp(text) => D( format!("D{}", &text[2..]) ),
            D(text) =>      DSharp( format!("D#{}", &text[1..]) ),
            DSharp(text) => E( format!("E{}", &text[2..]) ),
            E(text) =>      F( format!("F{}", &text[1..]) ),
            F(text) =>      FSharp( format!("F#{}", &text[1..]) ),
            FSharp(text) => G( format!("G{}", &text[2..]) ),
            G(text) =>      GSharp( format!("G#{}", &text[1..]) ),
            GSharp(text) => A( format!("A{}", &text[2..]) )
        }
    }
    fn do_step_left(&mut self) {
        *self = match self {
            A(text) =>      GSharp( format!("G#{}", &text[1..]) ),
            ASharp(text) => A( format!("A{}", &text[2..]) ),
            B(text) =>      ASharp( format!("A#{}", &text[1..]) ),
            C(text) =>      B( format!("B{}", &text[1..]) ),
            CSharp(text) => C( format!("C{}", &text[2..]) ),
            D(text) =>      CSharp( format!("C#{}", &text[1..]) ),
            DSharp(text) => D( format!("D{}", &text[2..]) ),
            E(text) =>      DSharp( format!("D#{}", &text[1..]) ),
            F(text) =>      E( format!("E{}", &text[1..]) ),
            FSharp(text) => F( format!("F{}", &text[2..]) ),
            G(text) =>      FSharp( format!("F#{}", &text[1..]) ),
            GSharp(text) => G( format!("G{}", &text[2..]) )
        }
    }
}
