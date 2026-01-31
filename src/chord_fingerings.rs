use crate::chord_fingerings::StringState::*;


const STRINGS: usize = 6;

pub struct Fingering {
    fret_num: u8,
    chord_size: u8,
    strings: [StringState; STRINGS]
}

pub enum StringState {
    Open,
    Muted,
    FrettedOn(u8)
}

impl Fingering {
    pub fn new(
        fret_num: u8,
        chord_size: u8,
        strings: [StringState; STRINGS],
    ) -> Self {
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
        text.push_str( &format!("  \n{} {}",
                if self.fret_num - 1 == 0 { "=".repeat(11) }
                else { "-".repeat(11) }
                , self.fret_num - 1) );
        if self.fret_num < 10 { text.push(' ') }
        text.push('\n');

        let mut fret_counter = self.fret_num;
        for i in 0..self.chord_size {
            text.push_str( &"| ".repeat(6) );
            text.push_str("  ");

            text.push('\n');
            for s in &strings {
                text.push_str(&
                    if let FrettedOn(f) = s && *f == (i + 1) { format!("{} ", f) }
                    else { "| ".to_string() }
                )
            }
            text.push_str("  \n");

            text.push_str( &"| ".repeat(6) );
            text.push_str("  \n");

            text.push_str( &"-".repeat(11) );
            text.push(' ');
            text.push_str(&fret_counter.to_string());
            if fret_counter < 10 { text.push(' ') }
            if (i + 1) < self.chord_size { text.push('\n') }

            fret_counter += 1;
        }

        // длина каждой строки 14 символов
        return text
    }
}
