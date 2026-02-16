use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use songbook::{Song, STANDART_TUNING};
use songbook::song::block;
use songbook::chord_generator::chord_fingerings::sum_text_in_fingerings;

use super::{
    TITLE_COLOR,
    CHORDS_COLOR,
    RHYTHM_COLOR,
    NOTES_COLOR,
};


pub fn get_as_paragraph<'a>(
    song: &'a Song,
    available_width: usize,
    needs_chords: bool,
    needs_rhythm: bool,
    needs_fingerings: bool, // дописать
    needs_notes: bool
) -> (Paragraph<'a>, usize, usize) {
    let mut lines = Vec::new();
    let mut columns = 0;

    if let Some(n) = &song.notes && !n.is_empty() && needs_notes {
        if n.chars().count() > columns { columns = n.chars().count() }
        lines.push( Line::styled(n, Style::new().fg(NOTES_COLOR)) );
        lines.push(Line::default());
    }

    if needs_chords && needs_fingerings {
        let mut fings = Vec::new();
        
        for chord in &song.chord_list {
            if let Ok(Some(f)) = songbook::song_library::get_fingering(&chord.text) {
                fings.push(f)
            } else {
                fings.push( chord.get_fingerings(&STANDART_TUNING)[0].clone() )
            }
        }

        
        if let Some(text) = sum_text_in_fingerings(&fings, Some(available_width)) {
            lines.extend( text.lines()
                .map(|l| Line::from(l.to_string()))
                .collect::<Vec<Line>>()
            );
        }
    }


    let mut is_first = true;
    for block in &song.blocks {
        if is_first { is_first = false }
        else { lines.push(Line::default()) }

        let mut head_block_spans = Vec::new();
        if let Some(title) = &block.title && !title.is_empty() {
            if title.chars().count() > columns { columns = title.chars().count() }
            head_block_spans.push(
                Span::styled(title.to_string() + " ", Style::new().fg(TITLE_COLOR))
            )
        }
        if let Some(n) = &block.notes && !n.is_empty() && needs_notes {
            if n.chars().count() > columns { columns = n.chars().count() }
            head_block_spans.push(
                Span::styled(n, Style::new().fg(NOTES_COLOR))
            )
        }
        if !head_block_spans.is_empty() { lines.push(Line::from(head_block_spans)) }


        for line in &block.lines {
            match line {
                block::Line::TextBlock(row) => {
                    let (chord_line, rhythm_line, text) = row.get_strings();
                    if !chord_line.is_empty() && needs_chords {
                        if chord_line.chars().count() > columns {
                            columns = chord_line.chars().count()
                        }
                        lines.push(Line::styled(chord_line, Style::new().fg(CHORDS_COLOR)))
                    }
                    if !rhythm_line.is_empty() && needs_rhythm {
                        if rhythm_line.chars().count() > columns {
                            columns = rhythm_line.chars().count()
                        }
                        lines.push(Line::styled(rhythm_line, Style::new().fg(RHYTHM_COLOR)))
                    }
                    if !text.is_empty() {
                        if text.chars().count() > columns {
                            columns = text.chars().count()
                        }
                        lines.push(Line::from(text))
                    }
                },
                block::Line::ChordsLine(chords) => if needs_chords {
                    let mut chord_line = String::new();
                    for chord in chords {
                        chord_line.push_str(&chord.text);
                        chord_line.push(' ');
                    }
                    if chord_line.chars().count() > columns { columns = chord_line.chars().count() }
                    lines.push(Line::styled(chord_line, Style::new().fg(CHORDS_COLOR)));
                },
                block::Line::PlainText(text) => {
                    lines.extend(text.lines()
                        .map(|l| Line::from(l))
                        .collect::<Vec<Line>>()
                    );
                    for l in text.lines() {
                        if l.chars().count() > columns { columns = l.chars().count() }
                    }
                },
                block::Line::EmptyLine => lines.push(Line::default())
            }
        }
    }

    let lines_len = lines.len();
    return (Paragraph::new(lines), lines_len, columns)
}
