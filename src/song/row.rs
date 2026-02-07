use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::song::chord::Chord;
use crate::{CHORDS_SYMBOL, RHYTHM_SYMBOL, TEXT_SYMBOL};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Beat {
    OnIndex{ index: usize, symbol: char },
    UpBeat(char)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Row {
    pub rhythm: Option<Vec<Beat>>,
    pub chords: Option<BTreeMap<usize, Chord>>, // позиция в строке - аккорд
    pub text: Option<String>
}

impl Row {
    pub fn to_string(&self, chords: bool, rhythm: bool) -> String {
        let mut s = String::new();
        let mut text = if let Some(t) = &self.text { t.clone() } else { String::new() };
        if chords || rhythm {
            let (c, r) = self.chords_and_rhythm_to_string(&mut text, chords, rhythm);
            if !c.is_empty() && chords {
                s.push_str(&c);
                s.push('\n');
            }

            if !r.is_empty() && rhythm {
                s.push_str(&r);
                s.push('\n');
            }
        }


        if !text.is_empty() {
            s.push_str(&text);
        }


        return s
    }
    fn chords_and_rhythm_to_string(
        &self,
        text: &mut String,
        needs_chords: bool,
        needs_rhythm: bool
    ) -> (String, String) {
        let mut chords_str = String::new();
        let (whitespaces, mut rhythm_str) =
            if let Some((w, r)) = self.get_rhythm_line() { (w, r) }
            else { (0, String::new()) };
        if !rhythm_str.is_empty() && rhythm_str.len() < text.len() {
            rhythm_str.push_str( &" ".repeat(text.len() - rhythm_str.len()) );
        }

        let mut added_indent = whitespaces;
        if let Some(chords) = &self.chords && needs_chords {
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
                            if let Some(b_index) = get_bytes_index_from_char_index(&text, p - whitespaces) {
                                match text.chars().nth(p - whitespaces) {
                                    Some(c) if c == ' ' => {
                                        text.insert_str(b_index, &" ".repeat(dif));
                                        if !rhythm_str.is_empty() {
                                            rhythm_str.insert_str(p - 1, &" ".repeat(dif));
                                        }
                                    },
                                    Some(_) => if let Some(prior_char) = text.chars().nth(p - 1 - whitespaces) {
                                        if prior_char == ' ' {
                                            text.insert_str(b_index, &" ".repeat(dif));
                                        } else {
                                            text.insert_str(b_index, &"-".repeat(dif));
                                        }

                                        if !rhythm_str.is_empty() {
                                            rhythm_str.insert_str(p - 1, &" ".repeat(dif));
                                        }
                                    },
                                    None => {
                                        text.push_str(&" ".repeat(dif));
                                        if !rhythm_str.is_empty() {
                                            rhythm_str.push_str(&" ".repeat(dif));
                                        }
                                    }
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
        }
        if whitespaces > 0 && needs_rhythm { text.insert_str(0, &" ".repeat(whitespaces)) }

        return (chords_str, rhythm_str)
    }


    pub fn get_for_editing(&self, s: &mut String) {
        let text = if let Some(t) = &self.text { t } else { &String::new() };
        let (whitespaces, rhythm_line) =
            if let Some(line) = self.get_rhythm_line() { line }
            else { ( 0, String::new() ) };

        let chords_line = if let Some(line) = self.get_chords_line() { line }
        else { String::new() };

        s.push_str(CHORDS_SYMBOL);
        s.push_str(&" ".repeat(whitespaces));
        s.push_str(&chords_line);
        s.push('\n');
        
        s.push_str(RHYTHM_SYMBOL);
        s.push_str(&rhythm_line);
        s.push('\n');

        s.push_str(TEXT_SYMBOL);
        s.push_str(&" ".repeat(whitespaces));
        s.push_str(&text);
        s.push('\n');
    }

    pub fn from_edited(text: &str) -> Self {
        let mut chord_line = String::new();
        let mut rhythm_line = String::new();
        let mut text_line = String::new();

        for line in text.lines() {
            if line.starts_with(CHORDS_SYMBOL) {
                chord_line.push_str(&line[CHORDS_SYMBOL.len()..])
            } else if line.starts_with(RHYTHM_SYMBOL) {
                rhythm_line.push_str(&line[RHYTHM_SYMBOL.len()..])
            } else if line.starts_with(TEXT_SYMBOL) {
                text_line.push_str(&line[TEXT_SYMBOL.len()..])
            }
        }

        let whitespaces = {
            let mut counter = 0;
            for c in text_line.chars() {
                if c == ' ' { counter += 1 }
                else { break }
            }

            counter
        };


        return Self {
            chords: chords_from_edited(&chord_line, whitespaces),
            rhythm: rhythm_from_edited(&rhythm_line, whitespaces),
            text: if text_line.is_empty() { None } else { Some(text_line.trim().to_string()) },
        }
    }


    pub fn get_chords_line(&self) -> Option<String> {
        let mut chords_str = String::new();
        let chords = if let Some(c) = &self.chords {c}
        else { return None };

        for k in chords.keys() {
            let i: usize;
            let p = k.saturating_sub(1);
            if chords_str.is_empty() {
                i = p;
            } else {
                let s_len = chords_str.chars().count();
                i = p - s_len;
            }

            chords_str.push_str(&" ".repeat(i));
            chords_str.push_str(&chords.get(k).unwrap().text);
        }


        return Some(chords_str)
    }

    fn get_rhythm_line(&self
    ) -> Option<(usize, String)> {
        let beats: &Vec<Beat> = if let Some(r) = &self.rhythm {r}
        else { return None };

        let mut whitespaces = 0;
        let mut added_indent = 0;
        let mut rhythm_line = String::new();
        for beat in beats {
            match beat {
                Beat::UpBeat(symbol) => {
                    whitespaces += 1 + 1; // один символ + один пробел
                    rhythm_line.push(*symbol);
                    rhythm_line.push(' ');
                },
                Beat::OnIndex { index, symbol } => {
                    let dif = index - added_indent;
                    rhythm_line.push_str(&" ".repeat(dif));
                    rhythm_line.push(*symbol);
                    rhythm_line.push(' ');
                    added_indent += dif + 2;
                }
            }
        }

        return Some( (whitespaces, rhythm_line) )
    }
}

fn get_bytes_index_from_char_index(line: &str, char_index: usize) -> Option<usize> {
    line.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
}
fn chords_from_edited(line: &str, whitespaces: usize) -> Option<BTreeMap<usize, Chord>> {
    let mut chords: BTreeMap<usize, Chord> = BTreeMap::new();

    let line = line.to_string() + " ";
    let mut chord = String::new();
    let mut indent = 0;
    for i in line.chars() {
        indent += 1;
        if i != ' ' {
            chord.push(i);
            continue
        } else if chord.is_empty() { continue }


        if let Some(c) = Chord::new(&chord) {
            chords.insert( indent - chord.chars().count() - whitespaces, c );
        }

        chord.clear();
    }


    if chords.is_empty() { None }
    else { Some(chords) }
}
fn rhythm_from_edited(line: &str, whitespaces: usize) -> Option<Vec<Beat>> {
    let mut beats: Vec<Beat> = Vec::new();
    for (i, c) in line.chars().enumerate() {
        if c != ' ' { beats.push(
            if whitespaces > i {
                Beat::UpBeat(c)
            } else {
                Beat::OnIndex {index: (i - whitespaces), symbol: c}
            }
        ) }
    }

    if beats.is_empty() { None }
    else { Some(beats) }
}

