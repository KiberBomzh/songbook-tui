use std::path::PathBuf;
use std::fs;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use ratatui::style::Color;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub colors: Colors
}

impl Config{
    pub fn new() -> Self {
        if let Ok(config) = Self::from_file() {
            return config
        } else {
            Self { colors: Colors::default() }
        }
    }

    fn from_file() -> Result<Self> {
        let path = Self::get_config_path().ok_or(anyhow::anyhow!("Cannot get config path!"))?;
        let content = fs::read_to_string(path)?;

        let config: Config = toml::from_str(&content)?;
        return Ok(config)
    }

    fn get_config_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("songbook");
        path.push("config.toml");
        if !path.is_file() { return None }
        return Some(path)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Colors {
    focus: String,
    unfocus: String,
    directories: String,
    songs: String,
    song: Song
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Song {
    title: String,
    chords: String,
    rhythm: String,
    notes: String,
    text: String
}
// Доступные цвета в Ratatui
// Black
// Red
// Green
// Yellow
// Blue
// Magenta
// Cyan
// Gray
// DarkGray
// LightRed
// LightGreen
// LightYellow
// LightBlue
// LightMagenta
// LightCyan
// White
// Rgb

impl Default for Colors {
    fn default() -> Self {
        Self {
            focus: "light_green".to_string(),
            unfocus: "dark_gray".to_string(),
            directories: "blues".to_string(),
            songs: "white".to_string(),
            song: Song {
                title: "green".to_string(),
                chords: "cyan".to_string(),
                rhythm: "yellow".to_string(),
                notes: "dark_gray".to_string(),
                text: "white".to_string()
            }
        }
    }
}

impl Colors {
    pub fn get_focus_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.focus) { Some(color) }
        else { None }
    }

    pub fn get_unfocus_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.unfocus) { Some(color) }
        else { None }
    }

    pub fn get_directories_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.directories) { Some(color) }
        else { None }
    }

    pub fn get_songs_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.songs) { Some(color) }
        else { None }
    }

    pub fn get_title_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.song.title) { Some(color) }
        else { None }
    }

    pub fn get_chords_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.song.chords) { Some(color) }
        else { None }
    }

    pub fn get_rhythm_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.song.rhythm) { Some(color) }
        else { None }
    }

    pub fn get_notes_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.song.notes) { Some(color) }
        else { None }
    }

    pub fn get_text_color(&self) -> Option<Color> {
        if let Ok(color) = Self::get_color(&self.song.text) { Some(color) }
        else { None }
    }

    fn get_color(color_str: &str) -> Result<Color> {
        if color_str.starts_with("rgb") {
            let rgb = &color_str[4..color_str.len() - 1];
            if let [r, g, b] = rgb
                .split(",")
                .map(|s| s.trim().parse::<u8>().unwrap_or(255))
                .collect::<Vec<u8>>()
                .as_slice()
            {
                Ok(Color::Rgb(*r, *g, *b))
            } else { Err(anyhow::anyhow!("Cannot get color!")) }
        } else {
            Ok(Color::from_str(color_str)?)
        }
    }
}

