use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::BTreeMap;

use crate::{Block, Row};


pub fn read_from_txt(file_path: &str) -> std::io::Result<Vec<Block>> {
    let mut blocks: Vec<Block> = Vec::new();

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut title = String::new();
    let mut rows: Vec<Row> = Vec::new();
    let mut chords = BTreeMap::new();
    let mut last_line_is_chords = false;
    for line_result in reader.lines() {
        let line = line_result?;
        if line.is_empty() {
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
            }
            continue
        };

        if is_line_chords(&line) {
            last_line_is_chords = true;

            let mut chord = String::new();
            let mut indent = 1;
            for i in line.chars() {
                if i == ' ' {
                    if !chord.is_empty() {
                        chords.insert(indent - chord.chars().count(), chord);
                        chord = String::new();
                    }
                    indent += 1;
                    continue
                }

                chord.push(i);
                indent += 1;
            }

        } else if last_line_is_chords {
            if chords.is_empty() {
                rows.push(Row { chords: None, text: Some(line)});
            } else {
                rows.push(Row {chords: Some(chords), text: Some(line)});
                chords = BTreeMap::new();
            }
            last_line_is_chords = false;
        } else if title.is_empty() {
            title = line;
        };
    }

    // Последний block
    if !rows.is_empty() {
        blocks.push(Block {
            title: if title.is_empty() { None } else { Some(title) },
            rows
        });
    }

    return Ok(blocks)
}

fn is_line_chords(line: &str) -> bool {
    let words: Vec<&str> = line.split_whitespace().collect();
    let chords = ["A", "B", "C", "D", "E", "F", "G"];

    words.iter().all(|w| chords.iter().any(|c| w.starts_with(*c)))
}
