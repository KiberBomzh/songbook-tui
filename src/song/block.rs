use serde::{Serialize, Deserialize};
use crate::song::row::Row;
use crate::{BLOCK_START, BLOCK_END, TITLE_SYMBOL, CHORDS_SYMBOL, RHYTHM_SYMBOL, TEXT_SYMBOL};


#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub title: Option<String>,
    pub rows: Vec<Row>,
}

impl Block {
    pub fn get_for_editing(&self, s: &mut String) {
        s.push_str(BLOCK_START);
        s.push('\n');

        s.push_str(TITLE_SYMBOL);
        if let Some(title) = &self.title {
            s.push_str(&title);
            if !self.rows.is_empty() { s.push('\n') }
        }

        let mut is_first_row = true;
        for row in &self.rows {
            if is_first_row { is_first_row = false }
            else { s.push('\n') }
            row.get_for_editing(s);
        }

        s.push_str(BLOCK_END);
        s.push('\n');
    }

    pub fn from_edited(text: &str) -> Self {
        let mut title: Option<String> = None;
        let mut rows: Vec<Row> = Vec::new();

        let mut row_buf = String::new();
        for line in text.lines() {
            if line.starts_with(TITLE_SYMBOL) {
                title = Some(line[TITLE_SYMBOL.len()..].to_string());
            } else if line.starts_with(CHORDS_SYMBOL) || line.starts_with(RHYTHM_SYMBOL) {
                row_buf.push_str(line);
                row_buf.push('\n');
            } else if line.starts_with(TEXT_SYMBOL) {
                row_buf.push_str(line);
                rows.push(Row::from_edited(&row_buf));
                row_buf.clear();
            }
        }

        return Self { title, rows }
    }
}
