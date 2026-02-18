mod file_reader;
pub mod chord_generator;
pub mod song;

#[cfg(feature = "song_library")]
pub mod song_library;

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crossterm::style::Color;

use crate::Note::*;
pub use crate::chord_generator::chord_fingerings::Fingering;
pub use crate::chord_generator::chord_fingerings::StringState::{self, *};
pub use crate::chord_generator::chord_fingerings::sum_text_in_fingerings;
pub use crate::chord_generator::get_fretboard;
pub use crate::chord_generator::STRINGS;
pub use crate::song::{Song, Metadata};
pub use crate::song::chord::Chord;


pub const STANDART_TUNING: [Note; STRINGS] = [E, B, G, D, A, E];


const METADATA_START: &str = "{metadata:}";
const METADATA_END: &str = "{:metadata}";

const SONG_TITLE_SYMBOL: &str = "{song_title:} ";
const SONG_ARTIST_SYMBOL: &str = "{song_artist:} ";
const SONG_KEY_SYMBOL: &str = "{song_key:} ";
const SONG_CAPO_SYMBOL: &str = "{song_capo:} ";


const BLOCK_START: &str = "{block:}";
const BLOCK_END: &str = "{:block}";

const TITLE_SYMBOL: &str = "{title:} ";

const CHORDS_LINE_SYMBOL: &str = "{chords_line:} ";
const EMPTY_LINE_SYMBOL: &str = "{empty_line}";

const PLAIN_TEXT_START: &str = "{plain_text:}";
const PLAIN_TEXT_END: &str = "{:plain_text}";

const CHORDS_SYMBOL: &str = "{C}|";
const RHYTHM_SYMBOL: &str = "{R}|";
const TEXT_SYMBOL: &str = "{T}|";

const SONG_NOTE_SYMBOL: &str = "{snote:} ";
const BLOCK_NOTE_SYMBOL: &str = "{bnote:} ";


const TITLE_COLOR: Color = Color::DarkGreen;
const CHORDS_COLOR: Color = Color::Cyan;
const RHYTHM_COLOR: Color = Color::Yellow;
const NOTES_COLOR: Color = Color::DarkGrey;


const KEYS: [[Note; 6]; 12] = [
    [C, D, E, F, G, A],
    [G, A, B, C, D, E],
    [D, E, FSharp, G, A, B],
    [A, B, CSharp, D, E, FSharp],
    [E, FSharp, GSharp, A, B, CSharp],
    [B, CSharp, DSharp, E, FSharp, GSharp],
    [FSharp, GSharp, ASharp, B, CSharp, DSharp],
    [CSharp, DSharp, F, FSharp, GSharp, ASharp],
    [GSharp, ASharp, C, CSharp, FSharp, F],
    [DSharp, F, G, GSharp, ASharp, C],
    [ASharp, C, D, DSharp, F, G],
    [F, G, A, ASharp, C, D]
];


#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
    
    pub fn get_key(text: &str) -> Option<Self> {
        match text {
            "C" | "c" | "Am" | "am" => Self::new("C"),
            "C#" | "c#" | "A#m" | "a#m"
                | "Db" | "db" | "Bbm" | "bbm" => Self::new("C#"),
            "D" | "d" | "Bm" | "bm" => Self::new("D"),
            "D#" | "d#" | "Cm" | "cm" | "Eb" | "eb" => Self::new("D#"),
            "E" | "e" | "C#m" | "c#m" | "Dbm" | "dbm" => Self::new("E"),
            "F" | "f" | "Dm" | "dm" => Self::new("F"),
            "F#" | "f#" | "D#m" | "d#m"
                | "Gb" | "gb" | "Ebm" | "ebm" => Self::new("F#"),
            "G" | "g" | "Em" | "em" => Self::new("G"),
            "G#" | "g#" | "Fm" | "fm" | "Ab" | "ab" => Self::new("G#"),
            "A" | "a" | "F#m" | "f#m" | "Gbm" | "gbm" => Self::new("A"),
            "A#" | "a#" | "Gm" | "gm" | "Bb" | "bb" => Self::new("A#"),
            "B" | "b" | "G#m" | "g#m" | "Abm" | "abm" => Self::new("B"),
            _ => None
        }
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


pub fn print_circle_of_fifth(needed_key: Option<Note>) {
    let mut s = String::new();
    let one_key_width = 18;
    let width = if let Ok( (cols, _rows) ) =  crossterm::terminal::size() {
        <u16 as Into<usize>>::into(cols)
    } else { one_key_width };
    let max_keys = width / one_key_width;

    let mut keys_already_in_line = 0;
    let mut keys_first_line = String::new();
    let mut keys_second_line = String::new();

    let mut keys = BTreeMap::new();
    for k in KEYS {
        let first = k[0].get_text();
        let second = k[1].get_text() + "m";
        let third = k[2].get_text() + "m";
        let fourth = k[3].get_text();
        let fifth = k[4].get_text();
        let sixth = k[5].get_text() + "m";
        
        let width: usize = 5;


        let mut first_line = String::new();
        first_line.push_str(&fourth);
        first_line.push_str( &" ".repeat(width - fourth.len()) );

        first_line.push_str(&first);
        first_line.push_str( &" ".repeat(width - first.len()) );

        first_line.push_str(&fifth);
        first_line.push_str( &" ".repeat(width - fifth.len()) );


        let mut second_line = String::new();
        second_line.push_str(&second);
        second_line.push_str( &" ".repeat(width - second.len()) );

        second_line.push_str(&sixth);
        second_line.push_str( &" ".repeat(width - sixth.len()) );

        second_line.push_str(&third);
        second_line.push_str( &" ".repeat(width - third.len()) );


        if keys_already_in_line < max_keys {
            keys_already_in_line += 1;

            keys_first_line.push_str(&first_line);
            keys_first_line.push_str("|  ");

            keys_second_line.push_str(&second_line);
            keys_second_line.push_str("|  ");
        } else {
            keys_already_in_line = 0;

            s.push_str(&keys_first_line);
            s.push('\n');

            s.push_str(&keys_second_line);
            s.push('\n');

            s.push_str( &"-".repeat((max_keys * one_key_width) - 2) );
            s.push('\n');

            keys_first_line.clear();
            keys_second_line.clear();
        }

        keys.insert(k[0], (first_line, second_line));
    }

    // Подтягивание последнего блока
    if !keys_first_line.is_empty() && !keys_second_line.is_empty() {
        s.push_str(&keys_first_line);
        s.push('\n');
        s.push_str(&keys_second_line);
        s.push('\n');

        keys_first_line.clear();
        keys_second_line.clear();
    }


    if let Some(needed_k) = needed_key {
        if let Some( (f_line, s_line) ) = keys.get(&needed_k) {
            println!("| {f_line}|\n| {s_line}|");
        }
    } else {
        println!("{s}");
    }
}
