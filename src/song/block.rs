use std::io::stdout;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor}
};
use serde::{Serialize, Deserialize};

use crate::song::row::Row;
use crate::song::chord::Chord;
use crate::{
    BLOCK_START,
    BLOCK_END,
    TITLE_SYMBOL,
    CHORDS_SYMBOL,
    RHYTHM_SYMBOL,
    TEXT_SYMBOL,
    EMPTY_LINE_SYMBOL,
    CHORDS_LINE_SYMBOL
};


#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub title: Option<String>,
    pub lines: Vec<Line>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Line {
    TextBlock(Row),
    ChordsLine(Vec<Chord>),
    EmptyLine
}

impl Line {
    pub fn print_colored(&self) {
        match self {
            Line::TextBlock(row) => row.print_colored(),
            Line::ChordsLine(chords) => {
                let mut s = String::new();
                for chord in chords {
                    s.push_str(&chord.text);
                    s.push(' ');
                }
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Magenta),
                    Print(s),
                    Print("\n"),
                    ResetColor
                ).unwrap_or(());
            },
            Line::EmptyLine => println!()
        }
    }
}

impl Block {
    pub fn get_for_editing(&self, s: &mut String) {
        s.push_str(BLOCK_START);
        s.push('\n');

        s.push_str(TITLE_SYMBOL);
        if let Some(title) = &self.title {
            s.push_str(&title);
        }
        if !self.lines.is_empty() { s.push('\n') }

        let mut is_first_row = true;
        for line in &self.lines {
            if is_first_row { is_first_row = false }
            else { s.push('\n') }
            match line {
                Line::TextBlock(row) => row.get_for_editing(s),
                Line::ChordsLine(chords) => {
                    s.push_str(CHORDS_LINE_SYMBOL);
                    for chord in chords {
                        s.push_str(&chord.text);
                        s.push(' ');
                    }
                    s.push('\n');
                },
                Line::EmptyLine =>  {
                    s.push_str(EMPTY_LINE_SYMBOL);
                    s.push('\n');
                }
            }
        }

        if !s.ends_with('\n') { s.push('\n') }
        s.push_str(BLOCK_END);
        s.push('\n');
    }

    pub fn from_edited(text: &str) -> Self {
        let mut title: Option<String> = None;
        let mut lines: Vec<Line> = Vec::new();

        let mut row_buf = String::new();
        for line in text.lines() {
            if line.starts_with(TITLE_SYMBOL) {
                title = Some(line[TITLE_SYMBOL.len()..].to_string());
            } else if line.starts_with(EMPTY_LINE_SYMBOL) {
                lines.push(Line::EmptyLine);
            } else if line.starts_with(CHORDS_LINE_SYMBOL) {
                let mut chords: Vec<Chord> = Vec::new();
                for maybe_chord in line.split_whitespace() {
                    if let Some(chord) = Chord::new(maybe_chord) {
                        chords.push(chord);
                    }
                }
                lines.push( Line::ChordsLine(chords) );
            } else if line.starts_with(CHORDS_SYMBOL) || line.starts_with(RHYTHM_SYMBOL) {
                row_buf.push_str(line);
                row_buf.push('\n');
            } else if line.starts_with(TEXT_SYMBOL) {
                row_buf.push_str(line);
                lines.push( Line::TextBlock(Row::from_edited(&row_buf)) );
                row_buf.clear();
            }
        }

        return Self { title, lines }
    }
}
