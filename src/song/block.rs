use crossterm::{
    execute,
    style::{Print, ResetColor, SetForegroundColor}
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
    CHORDS_LINE_SYMBOL,
    PLAIN_TEXT_START,
    PLAIN_TEXT_END,
    
    CHORDS_COLOR
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
    PlainText(String),
    EmptyLine
}

impl Line {
    pub fn print_colored(&self, out: &mut impl std::io::Write, needs_chords: bool, needs_rhythm: bool) -> std::io::Result<()> {
        match self {
            Line::TextBlock(row) => row.print_colored(out, needs_chords, needs_rhythm)?,
            Line::ChordsLine(chords) => if needs_chords {
                let mut s = String::new();
                for chord in chords {
                    s.push_str(&chord.text);
                    s.push(' ');
                }
                execute!(
                    out,
                    SetForegroundColor(CHORDS_COLOR),
                    Print(s),
                    Print("\n"),
                    ResetColor
                )?;
            },
            Line::PlainText(text) => write!(out, "{}", text)?,
            Line::EmptyLine => writeln!(out)?
        }
        
        Ok(())
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
                Line::PlainText(text) => {
                    s.push_str(PLAIN_TEXT_START);
                    s.push('\n');
                    
                    s.push_str(text);
                    s.push('\n');
                    
                    s.push_str(PLAIN_TEXT_END);
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


        let mut is_plain_text = false;
        let mut plain_text_buf = String::new();
        
        let mut row_buf = String::new();
        for line in text.lines() {
            if line.starts_with(PLAIN_TEXT_END) {
                is_plain_text = false;
                lines.push( Line::PlainText(plain_text_buf) );
                plain_text_buf = String::new();
            } else if is_plain_text {
                if !plain_text_buf.is_empty() {
                    plain_text_buf.push('\n');
                }
                plain_text_buf.push_str(line);
            } else if line.starts_with(PLAIN_TEXT_START) {
                is_plain_text = true;
            
            } else if line.starts_with(TITLE_SYMBOL) {
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
