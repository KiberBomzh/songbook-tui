mod file_reader;
mod chord_generator;

#[cfg(feature = "song_library")]
pub mod song_library;

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use crate::Note::*;
use crate::chord_generator::get_fingerings;
use crate::chord_generator::chord_fingerings::Fingering;

pub use crate::chord_generator::chord_fingerings::StringState::{self, *};
pub use crate::chord_generator::chord_fingerings::sum_text_in_fingerings;
pub use crate::chord_generator::get_fretboard;
pub use crate::chord_generator::STRINGS;


pub const STANDART_TUNING: [Note; STRINGS] = [E, B, G, D, A, E];

#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    pub metadata: Metadata,
    chord_list: Vec<Chord>,
    blocks: Vec<Block>,
    key: Note
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

impl Song {
    pub fn print(&self) {
        let mut fings = Vec::new();
        for f in self.get_fingerings() {
            fings.push(f[0].clone());
        }


        if !self.metadata.artist.is_empty() && !self.metadata.title.is_empty() {
            println!("{} - {}", self.metadata.artist, self.metadata.title);
        }

        if let Some(text) = sum_text_in_fingerings(&fings) {
            println!("{text}");
        }

        println!("{}", self.get_text());
    }

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

        return s
    }

    pub fn transpose(&mut self, steps: i32) {
        self.key = self.key.transpose(steps);
        for chord in &mut self.chord_list { *chord = chord.transpose(steps) }
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

    pub fn get_fingerings(&self) -> Vec<Vec<Fingering>> {
        let mut fings = Vec::new();
        for chord in &self.chord_list {
            fings.push(chord.get_fingerings(&STANDART_TUNING));
        }

        return fings
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
}

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
                chords_str.push_str(&chords.get(k).unwrap().text);
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



#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ChordType {
    Norm,
    Power,
    Sixth,
    SixthMinus,
    Seventh,
    MajSeventh,
    Nineth,
    Eleventh,
    Thirteenth
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum FifthState {
    Dim,
    Norm,
    Aug
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum SusOrAdd {
    No,
    Sus2,
    Sus4,
    Sus4Plus,
    Add2,
    Add4
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Chord {
    text: String,
    keynote: Note,
    minor: bool,
    chord_type: ChordType,
    fifth_state: FifthState,
    sus_or_add: SusOrAdd,
}

impl Chord {
    pub fn new(text: &str) -> Option<Self> {
        let (keynote, key_text) = match text {
            text if text.starts_with("A#") || text.starts_with("Bb") => (ASharp, "A#"),
            text if text.starts_with("C#") || text.starts_with("Db") => (CSharp, "C#"),
            text if text.starts_with("D#") || text.starts_with("Eb") => (DSharp, "D#"),
            text if text.starts_with("F#") || text.starts_with("Gb") => (FSharp, "F#"),
            text if text.starts_with("G#") || text.starts_with("Ab") => (GSharp, "G#"),
            text if text.starts_with('A') =>  (A, "A"),
            text if text.starts_with('B') =>  (B, "B"),
            text if text.starts_with('C') =>  (C, "C"),
            text if text.starts_with('D') =>  (D, "D"),
            text if text.starts_with('E') =>  (E, "E"),
            text if text.starts_with('F') =>  (F, "F"),
            text if text.starts_with('G') =>  (G, "G"),
            _ => return None
        };

        let text_after_key = &text[key_text.len()..];
        let text = text.to_string();
        let minor = text_after_key.starts_with('m') && !text_after_key.starts_with("maj");

        let fifth_state =
            if text_after_key.contains("aug") ||
                text_after_key.contains("5#") ||
                text_after_key.contains("5+") ||
                text_after_key.contains("+5") { FifthState::Aug }

            else if text_after_key.contains("dim") ||
                text_after_key.contains("5b") ||
                text_after_key.contains("5-") ||
                text_after_key.contains("-5") { FifthState::Dim }

            else { FifthState::Norm };


        let sus_or_add =
            // если третью ступень поднять ещё выше
            if text_after_key.starts_with("sus4+") ||
                text_after_key.starts_with("sus4#") { SusOrAdd::Sus4Plus }

            else if text_after_key.starts_with("sus2") { SusOrAdd::Sus2 }
            else if text_after_key.starts_with("sus4") { SusOrAdd::Sus4 }
            else if text_after_key.contains("add2") { SusOrAdd::Add2 }
            else if text_after_key.contains("add4") { SusOrAdd::Add4 }
            else { SusOrAdd::No };


        if text_after_key == "5" {
            return Some( Self { text, keynote, fifth_state, sus_or_add, 
                minor: false,
                chord_type: ChordType::Power,
            } )
        } else if text_after_key.contains("9") {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::Nineth
            } )
        } else if text_after_key.contains("11") {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::Eleventh
            } )
        } else if text_after_key.contains("13") {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::Thirteenth
            } )
        } else if text_after_key.contains("maj") {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::MajSeventh
            } )
        } else if text_after_key.contains('7') {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::Seventh
            } )
        } else if text_after_key.contains("6-") || text_after_key.contains("6b") {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::SixthMinus
            } )
        } else if text_after_key.contains('6') {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::Sixth
            } )
        } else {
            return Some( Self { text, keynote, minor, fifth_state, sus_or_add,
                chord_type: ChordType::Norm
            } )
        }
    }

    pub fn get_fingerings(&self, tuning: &[Note; STRINGS]) -> Vec<Fingering> {
        let mut notes: Vec<Note> = Vec::new();
        let key = self.keynote;
        // добавление первой ступени
        notes.push(key);


        // третья ступень
        if self.chord_type == ChordType::Power {
            notes.push( key.transpose(7) );
            return get_fingerings( tuning, &notes, Some(self.text.clone()) )
        }
        
        if self.sus_or_add == SusOrAdd::Sus2 {
            notes.push( key.transpose(2) );
        } else if self.sus_or_add == SusOrAdd::Sus4 {
            notes.push( key.transpose(5) );
        } else if self.sus_or_add == SusOrAdd::Sus4Plus {
            notes.push( key.transpose(6) );
        } else if self.minor {
            notes.push( key.transpose(3) );
        } else {
            notes.push( key.transpose(4) );
        }


        // пятая ступень
        match self.fifth_state {
            FifthState::Dim => notes.push( key.transpose(6) ),
            FifthState::Norm => notes.push( key.transpose(7) ),
            FifthState::Aug => notes.push( key.transpose(8) )
        }


        // дополнительные ноты
        if self.sus_or_add == SusOrAdd::Add2 {
            notes.push( key.transpose(2) );
        } else if self.sus_or_add == SusOrAdd::Add4 {
            notes.push( key.transpose(5) );
        } else if self.chord_type != ChordType::Norm {
            match self.chord_type {
                ChordType::SixthMinus => notes.push( key.transpose(8) ),
                ChordType::Sixth => notes.push( key.transpose(9) ),
                ChordType::Seventh => notes.push( key.transpose(10) ),
                ChordType::MajSeventh => notes.push( key.transpose(11) ),
                ChordType::Nineth => {
                    notes.push( key.transpose(2) );
                    notes.push( key.transpose(10) );
                },
                ChordType::Eleventh => {
                    notes.push( key.transpose(2) );
                    notes.push( key.transpose(5) );
                    notes.push( key.transpose(10) );
                },

                ChordType::Thirteenth => {
                    notes.push( key.transpose(2) );
                    notes.push( key.transpose(5) );
                    notes.push( key.transpose(9) );
                    notes.push( key.transpose(10) );
                },
                _ => {}

            }
        }


        return get_fingerings( tuning, &notes, Some(self.text.clone()) )
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
        (self.keynote, self.text) = match self.keynote {
            A =>      (ASharp, format!("A#{}", &self.text[1..]) ),
            ASharp => (B, format!("B{}", &self.text[2..]) ),
            B =>      (C, format!("C{}", &self.text[1..]) ),
            C =>      (CSharp, format!("C#{}", &self.text[1..]) ),
            CSharp => (D, format!("D{}", &self.text[2..]) ),
            D =>      (DSharp, format!("D#{}", &self.text[1..]) ),
            DSharp => (E, format!("E{}", &self.text[2..]) ),
            E =>      (F, format!("F{}", &self.text[1..]) ),
            F =>      (FSharp, format!("F#{}", &self.text[1..]) ),
            FSharp => (G, format!("G{}", &self.text[2..]) ),
            G =>      (GSharp, format!("G#{}", &self.text[1..]) ),
            GSharp => (A, format!("A{}", &self.text[2..]) )
        }
    }
    fn do_step_left(&mut self) {
        (self.keynote, self.text) = match self.keynote {
            A =>      (GSharp, format!("G#{}", &self.text[1..]) ),
            ASharp => (A, format!("A{}", &self.text[2..]) ),
            B =>      (ASharp, format!("A#{}", &self.text[1..]) ),
            C =>      (B, format!("B{}", &self.text[1..]) ),
            CSharp => (C, format!("C{}", &self.text[2..]) ),
            D =>      (CSharp, format!("C#{}", &self.text[1..]) ),
            DSharp => (D, format!("D{}", &self.text[2..]) ),
            E =>      (DSharp, format!("D#{}", &self.text[1..]) ),
            F =>      (E, format!("E{}", &self.text[1..]) ),
            FSharp => (F, format!("F{}", &self.text[2..]) ),
            G =>      (FSharp, format!("F#{}", &self.text[1..]) ),
            GSharp => (G, format!("G{}", &self.text[2..]) )
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Note {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp
}

impl Note {
    pub fn new(s: &str) -> Option<Self> {
        Some( match s {
            "A#" | "Bb" => ASharp,
            "C#" | "Db" => CSharp,
            "D#" | "Eb" => DSharp,
            "F#" | "Gb" => FSharp,
            "G#" | "Ab" => GSharp,

            "B" | "Cb" => B,
            "E" | "Fb" => E,

            "A" => A,
            "C" => C,
            "D" => D,
            "F" => F,
            "G" => G,
            _ => return None
        } )
    }

    pub fn get_text(&self) -> String {
        (
            match self {
                A      => "A",
                ASharp => "A#",
                B      => "B",
                C      => "C",
                CSharp => "C#",
                D      => "D",
                DSharp => "D#",
                E      => "E",
                F      => "F",
                FSharp => "F#",
                G      => "G",
                GSharp => "G#",
            }
        ).to_string()
    }

    pub fn transpose(&self, steps: i32) -> Self {
        let steps = steps % 12;
        if steps == 0 { return self.clone() }
        let mut note = self.clone();

        if steps > 0 {
            for _ in 0..steps { note.increase() }
        } else if steps < 0 {
            for _ in steps..0 { note.decrease() }
        }

        return note
    }
    fn increase(&mut self) {
        *self = match self {
            A =>      ASharp,
            ASharp => B,
            B =>      C,
            C =>      CSharp,
            CSharp => D,
            D =>      DSharp,
            DSharp => E,
            E =>      F,
            F =>      FSharp,
            FSharp => G,
            G =>      GSharp,
            GSharp => A
        }
    }

    fn decrease(&mut self) {
        *self = match self {
            A =>      GSharp,
            ASharp => A,
            B =>      ASharp,
            C =>      B,
            CSharp => C,
            D =>      CSharp,
            DSharp => D,
            E =>      DSharp,
            F =>      E,
            FSharp => F,
            G =>      FSharp,
            GSharp => G
        }
    }
}



pub fn print_fretboard(tuning: &[Note; STRINGS]) {
    let fretboard = crate::chord_generator::get_fretboard(tuning);
    let mut s = String::new();

    let note_width = 4;
    let line_width = fretboard.len() * note_width;
    let string_line = "|   ".repeat(fretboard.len());
    for fret_num in 0..fretboard[0].len() {
        if fret_num != 0 {
            s.push_str(&string_line);
            s.push('\n');
        } else { s.push('\n') }

        for string_num in (0..fretboard.len()).rev() {
            let note = &fretboard[string_num][fret_num].get_text();
            s.push_str(note);
            s.push_str( &" ".repeat(note_width - note.len()) );

        }

        if fret_num != 0 {
            s.push('\n');
            s.push_str(&string_line);
            s.push('\n');

            s.push_str( &"-".repeat(line_width - (note_width - 1)) );
            s.push(' ');
            s.push_str(&fret_num.to_string());
            s.push('\n');
        } else {
            s.push('\n');
            s.push_str( &"=".repeat(line_width - (note_width - 1)) );
            s.push('\n');
        }
    }

    println!("{s}");
}
