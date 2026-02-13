mod file_reader;
mod chord_generator;
mod song;

#[cfg(feature = "song_library")]
pub mod song_library;

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

const BLOCK_START: &str = "{block start}";
const BLOCK_END: &str = "{block end}";

const TITLE_SYMBOL: &str = "{title:} ";

const CHORDS_LINE_SYMBOL: &str = "{chords_line:} ";
const EMPTY_LINE_SYMBOL: &str = "{empty_line}";

const PLAIN_TEXT_START: &str = "{plain_text:}";
const PLAIN_TEXT_END: &str = "{:plain_text}";

const CHORDS_SYMBOL: &str = "{C}|";
const RHYTHM_SYMBOL: &str = "{R}|";
const TEXT_SYMBOL: &str = "{T}|";


const TITLE_COLOR: Color = Color::DarkGreen;
const CHORDS_COLOR: Color = Color::Cyan;
const RHYTHM_COLOR: Color = Color::Yellow;


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
