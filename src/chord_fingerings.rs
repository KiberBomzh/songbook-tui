use crate::chord_fingerings::StringState::*;


pub const STRINGS: usize = 6;

#[derive(Debug, PartialEq)]
pub struct Fingering {
    fret_num: u8,
    chord_size: u8,
    strings: [StringState; STRINGS]
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StringState {
    Open,
    Muted,
    FrettedOn(u8)
}

impl Fingering {
    pub fn new(strings: [StringState; STRINGS]) -> Self {
        let mut fret_num: u8 = 25;
        for s in &strings {
            if let FrettedOn(f) = s {
                if *f < fret_num { fret_num = *f }
            }
        }

        let mut chord_size: u8 = 0;
        for s in &strings {
            if let FrettedOn(f) = s {
                let current_size: u8 = (f + 1) - fret_num;
                if chord_size < current_size { chord_size = current_size }
            }
        }

        Self {
            fret_num,
            chord_size,
            strings
        }
    }

    pub fn get_text(&self) -> String {
        let mut text = String::new();

        let strings: Vec<&StringState> = self.strings.iter().rev().collect();
        // нулевой проход для отрисовки вначале открытых и заглушенных струн
        for s in &strings {
            text.push_str(match s {
                Open => "O ",
                Muted => "X ",
                FrettedOn(_) => "  "
            })
        }
        text.push_str( &format!("  \n{}   ",
                if self.fret_num - 1 == 0 { "=".repeat(11) }
                else { "-".repeat(11) }));
        text.push('\n');

        let mut fret_counter = self.fret_num;
        for i in 0..self.chord_size {
            text.push_str( &"| ".repeat(6) );
            text.push_str("  ");

            text.push('\n');
            for s in &strings {
                text.push_str(&
                    if let FrettedOn(f) = s && *f == (i + self.fret_num) { "o ".to_string() }
                    else { "| ".to_string() }
                )
            }
            text.push_str(&fret_counter.to_string());
            if fret_counter < 10 { text.push(' ') }
            text.push('\n');

            text.push_str( &"| ".repeat(6) );
            text.push_str("  \n");

            text.push_str( &"-".repeat(11) );
            text.push_str("   ");
            if (i + 1) < self.chord_size { text.push('\n') }

            fret_counter += 1;
        }

        // длина каждой строки 14 символов
        return text
    }
}
