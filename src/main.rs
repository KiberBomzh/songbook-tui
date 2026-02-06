#[cfg(feature = "tui")]
mod tui;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use songbook::{Song, Metadata, Note, STRINGS};
use songbook::song_library;


#[derive(Parser, Debug)]
#[command(name = "songbook")]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Print fretboard, for non-standart tuning use '-t'
    Fret {
        #[arg(short, long, default_value = "E, B, G, D, A, E", value_name = "TUNING")]
        tuning: String
    },

    /// Print chord's fingerings
    Chord { chord: String },

    /// Show song
    Show {
        path: PathBuf,
        
        /// Show in certain key
        #[arg(short, long)]
        key: Option<String>,

        /// Show chords
        #[arg(short, long)]
        chords: bool,

        /// Show rhythm
        #[arg(short, long)]
        rhythm: bool,
    },

    /// Edit song
    Edit {
        path: PathBuf,
        
        /// Target for editing
        #[arg(short, long)]
        target: EditTarget,
    },

    // Потом разбить это на разные варианты откуда добавлять
    // text, chordpro или создание пустой песни
    // добавить вариант с созданием песни с буфера обмена (text но с буфера)
    /// Add a song to the library
    Add {
        path: PathBuf,

        /// Song's artist
        #[arg(long, short)]
        artist: String,

        /// Song's title
        #[arg(long, short)]
        title: String,
    },

    /// Remove a song from the library
    Rm { paths: Vec<PathBuf> },

    /// Move(or rename) a song or a dir
    Mv { input_paths: Vec<PathBuf>, output_path: PathBuf },

    /// Print songs from the library
    Ls { path: Option<PathBuf> },

    /// Print tree
    Tree { path: Option<PathBuf> },

    /// Create directory in the library
    Mkdir { path: PathBuf },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum EditTarget {
    Song,
    Meta
}


fn main() {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Command::Fret { tuning } => {
                // check tuning
                let mut notes = [Note::A; STRINGS];
                let mut counter = 0;
                for n in tuning.split(", ") {
                    if let Some(note) = Note::new(&n) {
                        notes[counter] = note;
                    } else {
                        println!("Unknown note: {n}!");
                        return
                    }
                    counter += 1;
                    if counter == 6 { break }
                }
                if counter != STRINGS {
                    println!("Notes must be {STRINGS}!");
                    return
                }

                songbook::print_fretboard(&notes);
            },
            Command::Chord { chord } => {
                if let Some(chord) = songbook::Chord::new(&chord) {
                    let fings = chord.get_fingerings(&songbook::STANDART_TUNING);
                    if let Some(text) = songbook::sum_text_in_fingerings(&fings) {
                        println!("{text}");
                    }
                } else {
                    println!("Unknown chord!");
                }
            },
            Command::Show { path, key, chords, rhythm } => {
                let key = if let Some(k) = key.as_deref() { match k {
                    "C" | "c" | "Am" | "am" => Note::new("C"),
                    "C#" | "c#" | "A#m" | "a#m"
                        | "Db" | "db" | "Bbm" | "bbm" => Note::new("C#"),
                    "D" | "d" | "Bm" | "bm" => Note::new("D"),
                    "D#" | "d#" | "Cm" | "cm" | "Eb" | "eb" => Note::new("D#"),
                    "E" | "e" | "C#m" | "c#m" | "Dbm" | "dbm" => Note::new("E"),
                    "F" | "f" | "Dm" | "dm" => Note::new("F"),
                    "F#" | "f#" | "D#m" | "d#m"
                        | "Gb" | "gb" | "Ebm" | "ebm" => Note::new("F#"),
                    "G" | "g" | "Em" | "em" => Note::new("G"),
                    "G#" | "g#" | "Fm" | "fm" | "Ab" | "ab" => Note::new("G#"),
                    "A" | "a" | "F#m" | "f#m" | "Gbm" | "gbm" => Note::new("A"),
                    "A#" | "a#" | "Gm" | "gm" | "Bb" | "bb" => Note::new("A#"),
                    "B" | "b" | "G#m" | "g#m" | "Abm" | "abm" => Note::new("B"),
                    _ => {
                        println!("Unknown key: {k}!");
                        None
                    }
                } } else { None };

                song_library::show(&path, key, chords, rhythm)
                    .expect("Error during geting song!");
            },
            Command::Edit { path, target } => {
                let target = match target {
                    EditTarget::Song => "song",
                    EditTarget::Meta => "meta"
                };
                song_library::edit(&path, target)
                    .expect("Error during editing song!");
            },
            Command::Add {path, artist, title} => {
                let song = Song::from_txt(
                    &path,
                    Metadata { title, artist, key: None }
                    ).expect("Error during adding a song!");

                song_library::add(&song)
                    .expect("Error during adding a song!");
            },
            Command::Rm { paths } => {
                for path in &paths {
                    song_library::rm(&path)
                        .expect("Error during removing!");
                }
            },
            Command::Mv {input_paths, output_path } => {
                for input_path in &input_paths {
                    song_library::mv(&input_path, &output_path)
                        .expect("Error during moving!");
                }
            },
            Command::Ls { path } => {
                song_library::ls(path.as_deref())
                    .expect("Error during reading a dir!");
            },
            Command::Tree { path } => {
                song_library::tree(path.as_deref())
                    .expect("Error during reading a dir!");
            },
            Command::Mkdir { path } => {
                song_library::mkdir(&path)
                    .expect("Error during creating a dir!");
            },
        }
    } else {
        #[cfg(not(feature = "tui"))]
        println!("There's a command required! Try 'songbook help' for more information");

        #[cfg(feature = "tui")]
        println!("TUI is still in development");
    }
}
