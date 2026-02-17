mod song_formater;

use std::path::PathBuf;
use anyhow::Result;

use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Paragraph, List, ListState, ListItem};
use ratatui::prelude::*;

use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyCode};

use songbook::song_library::lib_functions::*;
use songbook::Song;

const TITLE_COLOR: Color = Color::Green;
const CHORDS_COLOR: Color = Color::Cyan;
const RHYTHM_COLOR: Color = Color::Yellow;
const NOTES_COLOR: Color = Color::DarkGray;


pub fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let (lib_list, current_dir) = get_files_in_dir(None)?;
    let mut app = App {
        exit: false,
        focus: Focus::Library,
        hide_lib: false,
        lib_list_state: ListState::default(),
        lib_list,
        current_dir,
        last_dirs: Vec::new(),
        current_song: None,
        show_chords: true,
        show_rhythm: true,
        show_fingerings: false,
        show_notes: true,
        scroll_y: 0,
        scroll_x: 0,
        scroll_y_max: 0,
        scroll_x_max: 0
    };
    app.lib_list_state.select(Some(0));

    let app_result = app.run(&mut terminal);

    ratatui::restore();

    return app_result
}


#[derive(PartialEq)]
enum Focus {
    Library,
    Song
}

struct App {
    exit: bool,

    focus: Focus,
    hide_lib: bool,

    lib_list_state: ListState,
    lib_list: Vec<(String, PathBuf)>,
    current_dir: PathBuf,
    last_dirs: Vec<PathBuf>,

    current_song: Option<(Song, PathBuf)>,
    show_chords: bool,
    show_rhythm: bool,
    show_fingerings: bool,
    show_notes: bool,

    scroll_y: u16,
    scroll_x: u16,
    scroll_y_max: usize,
    scroll_x_max: usize
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match crossterm::event::read()? {
                Event::Key(key_event) => self.handle_key_event(key_event, terminal)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        use Constraint::Percentage;

        let horizontal = if self.hide_lib {
            Layout::horizontal([Percentage(0), Percentage(100)])
        } else {
            Layout::horizontal([Percentage(25), Percentage(75)])
        };
        let [lib_area, song_area] = horizontal.areas(frame.area());

        let focus_color = Color::White;
        let unfocus_color = Color::DarkGray;


        let items: Vec<ListItem> = self.lib_list.iter()
            .map(|(s, f)|
                if f.is_dir() { ListItem::new(s.as_str()).style(
                    Style::new().blue()
                )}
                else { ListItem::new(s.as_str()) }
            )
            .collect();

        if !self.hide_lib {
            let list = List::new(items)
                .highlight_style(Style::new())
                .highlight_symbol("->")
                .block(
                    Block::bordered().title("Library")
                        .border_style(if self.focus == Focus::Library {
                            Style::new().fg(focus_color)
                        } else {
                            Style::new().fg(unfocus_color)
                        })
                );
            frame.render_stateful_widget(list, lib_area, &mut self.lib_list_state);
        }


        let song_block = Block::bordered()
            .border_style(if self.focus == Focus::Song {
                Style::new().fg(focus_color)
            } else {
                Style::new().fg(unfocus_color)
            });
        let inner_song_area = song_block.inner(song_area);

        let title: String;
        let song = if let Some((song, _p)) = &self.current_song {
            title = format!("{} - {}", song.metadata.artist, song.metadata.title);

            let height = <u16 as Into<usize>>::into(inner_song_area.height);
            let width = <u16 as Into<usize>>::into(inner_song_area.width);

            let (p, lines, columns) = song_formater::get_as_paragraph(
                &song,
                width,
                self.show_chords,
                self.show_rhythm,
                self.show_fingerings,
                self.show_notes
            );

            self.scroll_y_max = lines.saturating_sub(height);
            self.scroll_x_max = columns.saturating_sub(width);

            p.scroll( (self.scroll_y, self.scroll_x) )
        } else {
            title = "Nothing to show".to_string();
            Paragraph::default()
        }.block(song_block.title(title));
        frame.render_widget(song, song_area);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        if key_event.kind == KeyEventKind::Press {
            if key_event.code == KeyCode::Char('q') { self.exit = true }
            else if key_event.code == KeyCode::Tab && !self.hide_lib { self.switch_focus() }
            else {
                match self.focus {
                    Focus::Library => self.handle_lib_key_event(key_event)?,
                    Focus::Song => self.handle_song_key_event(key_event, terminal)?
                }
            }
        }
        Ok(())
    }

    fn handle_lib_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => self.lib_list_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.lib_list_state.select_previous(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if let Some(selected) = self.lib_list_state.selected() {
                    let (_name, path) = &self.lib_list[selected];
                    if path.is_dir() {
                        self.last_dirs.push(self.current_dir.clone());
                        (self.lib_list, self.current_dir) = get_files_in_dir(Some(&path))?;
                        self.lib_list_state.select_first();
                    } else if path.is_file() {
                        self.current_song = Some( (get_song(&path)?, path.to_path_buf()) );
                        self.focus = Focus::Song;
                        self.scroll_y = 0;
                        self.scroll_x = 0;
                    }
                }
            },
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                (self.lib_list, self.current_dir) =
                    get_files_in_dir( self.last_dirs.pop().as_deref() )?;
                self.lib_list_state.select_first();
            },


            KeyCode::Char('S') => {
                songbook::song_library::sort()?;
                self.last_dirs.clear();
                (self.lib_list, self.current_dir) = get_files_in_dir(None)?;
                self.lib_list_state.select_first();
            },
            _ => {}
        }

        Ok(())
    }

    fn handle_song_key_event(
        &mut self,
        key_event: KeyEvent,
        terminal: &mut DefaultTerminal
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down =>
                if self.scroll_y_max > self.scroll_y.into() { self.scroll_y += 1 },

            KeyCode::Char('k') | KeyCode::Up =>
                self.scroll_y = self.scroll_y.saturating_sub(1),

            KeyCode::Char('l') | KeyCode::Right =>
                if self.scroll_x_max > self.scroll_x.into() { self.scroll_x += 1 },

            KeyCode::Char('h') | KeyCode::Left =>
                self.scroll_x = self.scroll_x.saturating_sub(1),


            KeyCode::Char('C') => {
                if self.show_chords {
                    self.show_chords = false
                } else {
                    self.show_chords = true
                }
            },
            KeyCode::Char('R') => {
                if self.show_rhythm {
                    self.show_rhythm = false
                } else {
                    self.show_rhythm = true
                }
            },
            KeyCode::Char('F') => {
                if self.show_fingerings {
                    self.show_fingerings = false
                } else {
                    self.show_fingerings = true
                }
            },
            KeyCode::Char('N') => {
                if self.show_notes {
                    self.show_notes = false
                } else {
                    self.show_notes = true
                }
            },
            
            KeyCode::Char(';') => self.switch_lib(),


            KeyCode::Char('E') => {
                if let Some( (_s, path) ) = &self.current_song {
                    ratatui::restore();

                    songbook::song_library::edit(path, "song")?;
                    self.current_song = Some( (get_song(path)?, path.to_path_buf()) );

                    *terminal = ratatui::init();
                }
            },
            _ => {}
        }

        Ok(())
    }

    fn switch_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Library => Focus::Song,
            Focus::Song => Focus::Library
        }
    }

    fn switch_lib(&mut self) {
        if self.hide_lib {
            self.hide_lib = false;
            self.focus = Focus::Library;
        } else {
            self.hide_lib = true
        }
    }
}
