mod song_formater;

use std::path::PathBuf;
use anyhow::Result;

use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Paragraph, List, ListState, ListItem};
use ratatui::prelude::*;

use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyCode};

use rfd::FileDialog;

use songbook::song_library::lib_functions::*;
use songbook::Song;

const TITLE_COLOR: Color = Color::Green;
const CHORDS_COLOR: Color = Color::Cyan;
const RHYTHM_COLOR: Color = Color::Yellow;
const NOTES_COLOR: Color = Color::DarkGray;

const FOCUS_COLOR: Color = Color::LightGreen;
const UNFOCUS_COLOR: Color = Color::DarkGray;



pub fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let (lib_list, current_dir) = get_files_in_dir(None)?;
    let mut app = App {
        exit: false,
        focus: Focus::Library,
        hide_lib: false,
        is_long_command: false,
        long_command: String::new(),
        lib_list_state: ListState::default(),
        lib_list,
        current_dir,
        last_dirs: Vec::new(),
        cutted_path: None,
        current_song: None,
        song_area_height: None,
        song_area_width: None,
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

    is_long_command: bool,
    long_command: String,

    lib_list_state: ListState,
    lib_list: Vec<(String, PathBuf)>,
    current_dir: PathBuf,
    last_dirs: Vec<PathBuf>,
    cutted_path: Option<PathBuf>,

    current_song: Option<(Song, PathBuf)>,
    song_area_height: Option<usize>,
    song_area_width: Option<usize>,
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


        let mut items: Vec<ListItem> = Vec::new();
        for (name, path) in &self.lib_list {
            let mut style = Style::new();
            if path.is_dir() { style = style.blue(); }
            if let Some(c_path) = &self.cutted_path && c_path == path { style = style.dim(); }
            items.push(ListItem::new(name.as_str()).style(style));
        }

        if !self.hide_lib {
            let list = List::new(items)
                .highlight_style(Style::new())
                .highlight_symbol("->")
                .block(
                    Block::bordered().title(
                        self.current_dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("library")
                        )

                        .border_style(if self.focus == Focus::Library {
                            Style::new().fg(FOCUS_COLOR)
                        } else {
                            Style::new().fg(UNFOCUS_COLOR)
                        })
                );
            frame.render_stateful_widget(list, lib_area, &mut self.lib_list_state);
        }


        let song_block = Block::bordered()
            .border_style(if self.focus == Focus::Song {
                Style::new().fg(FOCUS_COLOR)
            } else {
                Style::new().fg(UNFOCUS_COLOR)
            });
        let inner_song_area = song_block.inner(song_area);

        let title: String;
        let title_top: String;
        let song = if let Some((song, _p)) = &self.current_song {
            title = format!("{} - {}", song.metadata.artist, song.metadata.title);

            let mut t_top_buf = String::new();
            if let Some(key) = &song.metadata.key {
                t_top_buf.push_str("Key: ");
                t_top_buf.push_str(&key.get_text());
            }
            if let Some(capo) = &song.metadata.capo {
                if !t_top_buf.is_empty() { t_top_buf.push_str(", ") }
                t_top_buf.push_str("Capo: ");
                t_top_buf.push_str(&capo.to_string());
            }
            title_top = t_top_buf;



            let height = <u16 as Into<usize>>::into(inner_song_area.height);
            let width = <u16 as Into<usize>>::into(inner_song_area.width);
            self.song_area_height = Some(height);
            self.song_area_width = Some(width);

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
            title_top = String::new();
            Paragraph::default()
        }.block(
            song_block
                .title(title)
                .title_bottom(Line::from(self.long_command.as_str()).right_aligned())
                .title_top(Line::from(title_top).right_aligned())
        );
        frame.render_widget(song, song_area);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut is_song_changed = false;
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char(c) if self.is_long_command => self.long_command.push(c),
                KeyCode::Backspace if self.is_long_command => {
                    self.long_command.pop();
                    if self.long_command.is_empty() { self.is_long_command = false }
                },
                KeyCode::Enter if self.is_long_command => {
                    if !self.long_command.is_empty() {
                        match self.focus {
                            Focus::Library => self.handle_long_command_in_library()?,
                            Focus::Song => self.handle_long_command_in_song(&mut is_song_changed)?
                        }
                    }

                    self.is_long_command = false;
                    self.long_command.clear();
                },


                KeyCode::Char('q') => self.exit = true,
                KeyCode::Tab => if !self.hide_lib { self.switch_focus() },
                _ => {
                    match self.focus {
                        Focus::Library => self.handle_lib_key_event(key_event)?,
                        Focus::Song => self.handle_song_key_event(key_event, terminal, &mut is_song_changed)?
                    }
                }
            }
        }

        if is_song_changed {
            if let Some( (song, path) ) = &self.current_song {
                save(song, path)?;
            }
        }

        Ok(())
    }

    fn handle_lib_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('n') |
            KeyCode::Char('r') |
            KeyCode::Char('f') |
            KeyCode::Char('a') => {
                self.is_long_command = true;
                if let KeyCode::Char(c) = key_event.code {
                    self.long_command.push(c)
                }
            },

            KeyCode::Char('c') => if let Some(selected) = self.lib_list_state.selected() {
                let (_name, path) = &self.lib_list[selected];
                self.cutted_path = Some(path.to_path_buf());
            },
            KeyCode::Char('p') => if let Some(path) = &self.cutted_path {
                songbook::song_library::mv(path, &self.current_dir)?;
                (self.lib_list, self.current_dir) = get_files_in_dir(Some(&self.current_dir))?;
                self.cutted_path = None;
            },

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

            KeyCode::Char('D') => {
                if let Some(selected) = self.lib_list_state.selected() {
                    let (_name, path) = &self.lib_list[selected];
                    songbook::song_library::rm(path)?;
                    (self.lib_list, self.current_dir) = get_files_in_dir( Some(&self.current_dir) )?;
                }
            },
            _ => {}
        }


        if let Some( (_s, path) ) = &self.current_song {
            if !path.is_file() { self.current_song = None }
        }

        Ok(())
    }

    fn handle_song_key_event(
        &mut self,
        key_event: KeyEvent,
        terminal: &mut DefaultTerminal,
        is_song_changed: &mut bool
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('T') | KeyCode::Char('C') => {
                self.is_long_command = true;
                if let KeyCode::Char(c) = key_event.code {
                    self.long_command.push(c)
                }
            },


            KeyCode::Char('j') | KeyCode::Down =>
                if self.scroll_y_max > self.scroll_y.into() { self.scroll_y += 1 },

            KeyCode::Char('k') | KeyCode::Up =>
                self.scroll_y = self.scroll_y.saturating_sub(1),

            KeyCode::Char('l') | KeyCode::Right =>
                if self.scroll_x_max > self.scroll_x.into() { self.scroll_x += 1 },

            KeyCode::Char('h') | KeyCode::Left =>
                self.scroll_x = self.scroll_x.saturating_sub(1),


            KeyCode::Char('J') | KeyCode::PageDown => {
                if let Some(height) = self.song_area_height {
                    let height: u16 = height.try_into()?;
                    if self.scroll_y_max > (self.scroll_y + height).into() {
                        self.scroll_y += height;
                    } else if self.scroll_y_max > self.scroll_y.into() {
                        self.scroll_y = self.scroll_y_max.try_into()?;
                    }
                }
            },
            KeyCode::Char('K') | KeyCode::PageUp => {
                if let Some(height) = self.song_area_height {
                    self.scroll_y = self.scroll_y.saturating_sub(height.try_into()?);
                }
            },

            KeyCode::Home => self.scroll_y = 0,
            KeyCode::End => self.scroll_y = self.scroll_y_max.try_into()?,


            KeyCode::Char('c') => {
                if self.show_chords {
                    self.show_chords = false
                } else {
                    self.show_chords = true
                }
            },
            KeyCode::Char('r') => {
                if self.show_rhythm {
                    self.show_rhythm = false
                } else {
                    self.show_rhythm = true
                }
            },
            KeyCode::Char('f') => {
                if self.show_fingerings {
                    self.show_fingerings = false
                } else {
                    self.show_fingerings = true
                }
            },
            KeyCode::Char('n') => {
                if self.show_notes {
                    self.show_notes = false
                } else {
                    self.show_notes = true
                }
            },
            
            KeyCode::Char(';') => self.switch_lib(),


            KeyCode::Char('e') => {
                if let Some( (song, _path) ) = &mut self.current_song {
                    ratatui::restore();
                    edit(song)?;
                    *is_song_changed = true;
                    *terminal = ratatui::init();
                }
            },
            _ => {}
        }

        Ok(())
    }

    fn handle_long_command_in_song(
        &mut self,
        is_song_changed: &mut bool
    ) -> Result<()> {
        let song = if let Some( (song, _p) ) = &mut self.current_song { song }
            else { return Ok(()) };

        let command = if let Some(c) = self.long_command.chars().next() { c }
            else { return Ok(()) };
        let command_data: String = self.long_command.chars().skip(1).collect();
        match command {
            'T' => {
                let steps: i32 = if let Ok(s) = command_data.parse() { s }
                    else { return Ok(()) };
                song.transpose(steps);
                *is_song_changed = true;
            },
            'C' => if let Ok(capo) = command_data.parse::<u8>() {
                if let Some(song_capo) = song.metadata.capo {
                    let song_capo: i32 = song_capo.into();
                    let capo: i32 = capo.into();
                    song.transpose(capo - song_capo);
                } else {
                    song.transpose(capo.into());
                }
                song.metadata.capo =
                    if capo == 0 { None }
                    else { Some(capo) };
                *is_song_changed = true;
            },
            _ => {}
        }

        Ok(())
    }

    fn handle_long_command_in_library(&mut self) -> Result<()> {
        let command = if let Some(c) = self.long_command.chars().next() { c }
            else { return Ok(()) };
        let command_data: String = self.long_command.chars().skip(1).collect();
        if command_data.is_empty() { return Ok(()) }
        match command {
            'n' => {
                songbook::song_library::mkdir(
                    &self.current_dir.join(command_data)
                )?;
                (self.lib_list, self.current_dir) = get_files_in_dir( Some(&self.current_dir) )?;
            },
            'r' => {
                if let Some(selected) = self.lib_list_state.selected() {
                    let (_name, path) = &self.lib_list[selected];
                    let parent_path = if let Some(p) = path.parent() { p }
                        else { &self.current_dir };
                    songbook::song_library::mv(path, &parent_path.join(command_data))?;
                    (self.lib_list, self.current_dir) = get_files_in_dir( Some(&self.current_dir) )?;
                }
            },
            'f' => {
                self.current_dir = songbook::song_library::get_lib_path()?;
                self.lib_list = find(&command_data)?;
            },
            'a' => {
                let subcommand = if let Some(c) = command_data.chars().nth(0) { c }
                    else { return Ok(()) };
                let meta: Option<(String, String)> = if let [artist, title, ..] =
                    command_data
                        .chars()
                        .skip(1)
                        .collect::<String>()
                        .split(" - ")
                        .collect::<Vec<&str>>()
                        .as_slice() { Some( (artist.trim().to_string(), title.trim().to_string()) ) }
                        else { None };

                let song: Option<Song> = match subcommand {
                    'e' => if let Some( (artist, title) ) = meta {
                        Some(Song::new(&title, &artist))
                    } else { None },
                    't' => if let Some( (artist, title) ) = meta {
                        if let Some(file) = FileDialog::new()
                            .add_filter("text", &["txt"])
                            .pick_file() {
                            Some(Song::from_txt(&file, &title, &artist)?)
                        } else { None }
                    } else { None },
                    'c' => {
                        if let Some(file) = FileDialog::new()
                            .add_filter("text", &["chordpo", "cho"])
                            .pick_file() {
                            let mut song = Song::from_chordpro(&file)?;
                            if let Some( (artist, title) ) = meta {
                                song.metadata.title = title;
                                song.metadata.artist = artist;
                            }

                            Some(song)
                        } else { None }
                    },
                    _ => None
                };

                
                if let Some(s) = &song {
                    songbook::song_library::add(s)?;
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
