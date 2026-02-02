use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::BTreeMap;

use crate::{Block, Row, Chord};


pub fn read_from_txt(file_path: &Path) -> std::io::Result<(Vec<Block>, Vec<Chord>)> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut chord_list: Vec<Chord> = Vec::new();

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut title = String::new();
    let mut rows: Vec<Row> = Vec::new();
    let mut chords = BTreeMap::new();
    let mut last_line_is_chords = false;
    let mut last_line_was_empty = true;
    for line_result in reader.lines() {
        let line = line_result?;
        if line.is_empty() {
            last_line_was_empty = true;
            if !rows.is_empty() {
                blocks.push(Block {
                    title: if title.is_empty() { None } else { Some(title) },
                    rows
                });
                title = String::new();
                rows = Vec::new();
            } else if last_line_is_chords {
                if !chords.is_empty() {
                    blocks.push(Block {
                        title: if title.is_empty() { None } else { Some(title) },
                        rows: vec!(Row { chords: Some(chords), text: None })
                    });
                    title = String::new();
                    chords = BTreeMap::new();
                    last_line_is_chords = false
                }
            } else if !title.is_empty() {
                blocks.push(Block {
                    title: Some(title),
                    rows: Vec::new()
                });
                title = String::new();
            }

            continue
        };

        if is_line_chords(&line) && !last_line_is_chords {
            last_line_is_chords = true;

            let line = line + " ";
            let mut chord = String::new();
            let mut indent = 1;
            for i in line.chars() {
                if i == ' ' {
                    if !chord.is_empty() {
                        if let Some(c) = Chord::new(&chord) {
                            chords.insert(indent - chord.chars().count(), c.clone());
                            if chord_list.iter().all(|chord| *chord != c) {
                                chord_list.push(c);
                            }
                        }

                        chord.clear();
                    }
                    indent += 1;
                    continue
                }

                chord.push(i);
                indent += 1;
            }

        } else if last_line_is_chords {
            if chords.is_empty() {
                rows.push(Row { chords: None, text: Some(line) });
            } else {
                rows.push(Row { chords: Some(chords), text: Some(line) });
                chords = BTreeMap::new();
            }
            last_line_is_chords = false;
        } else if last_line_was_empty {
            title = line;
        } else {
            rows.push(Row { chords: None, text: Some(line) });
        }

        last_line_was_empty = false;
    }

    // Последний block
    if !rows.is_empty() {
        blocks.push(Block {
            title: if title.is_empty() { None } else { Some(title) },
            rows
        });
    }

    return Ok((blocks, chord_list))
}

fn is_line_chords(line: &str) -> bool {
    let words: Vec<&str> = line.split_whitespace().collect();
    let chords = ["A", "B", "C", "D", "E", "F", "G"];

    words.iter().all(|w| chords.iter().any(|c| w.starts_with(*c)))
}
