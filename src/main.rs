#[cfg(feature = "tui")]
mod tui;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use songbook::{Song, Metadata, Note, STRINGS};
use songbook::song_library;
use songbook::{Fingering, StringState};


#[derive(Parser, Debug)]
#[command(name = "songbook")]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Create folder for songs and add couple songs-examples
    Init,

    /// Print fretboard, for non-standart tuning use '-t'
    Fret {
        #[arg(short, long, default_value = "E, B, G, D, A, E", value_name = "TUNING")]
        tuning: String
    },

    /// Print chord's fingerings
    Chord { chord: String },
    
    /// Manage your fingerings
    Fingering {
        /// Your chord name
        #[arg(long, short)]
        chord: String,

        /// Strings' conditions (x - muted, 0 - open, 1-24 - fretted)
        #[arg(long, short, num_args = 1..)]
        fingering: Vec<String>,
    },


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
        
        /// Show fingerings
        #[arg(short, long)]
        fingerings: bool,
        
        /// Use colored chords and rhythm
        #[arg(long)]
        colored: bool,
    },

    /// Edit song
    Edit {
        path: PathBuf,
        
        /// Target for editing
        #[arg(short, long)]
        target: EditTarget,
    },

    // добавить вариант с созданием песни с буфера обмена (text но с буфера)
    /// Add a song to the library
    #[command(subcommand)]
    Add(AddSubcommand),

    /// Remove a song from the library
    Rm { paths: Vec<PathBuf> },

    /// Move(or rename) a song or a dir
    Mv {
        /// Input files for
        #[arg(num_args = 1.., required = true)]
        input_paths: Vec<PathBuf>,
        output_path: PathBuf
    },

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

#[derive(Subcommand, Debug, Clone)]
enum AddSubcommand {
    FromTxt {
        path: PathBuf,

        /// Song's artist
        #[arg(long, short)]
        artist: String,

        /// Song's title
        #[arg(long, short)]
        title: String,
    },
    // FromChordPro { path: PathBuf },
    Empty {
        /// Song's artist
        #[arg(long, short)]
        artist: String,

        /// Song's title
        #[arg(long, short)]
        title: String,
    }
}


fn main() {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Command::Init => song_library::init().expect("Error during initialisation!"),
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
            Command::Fingering { chord, fingering } => {
                if fingering.len() != STRINGS {
                    println!("Len --fingering must be {}!", STRINGS);
                    return
                }
                let allowed = ["x", "0", 
                    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
                    "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24"
                ];
                if !fingering.iter().all(|f|
                    allowed.iter().any(|a| a == f)
                ) {
                    println!("There's not allowed char in your fingering!");
                    return
                }
                
                let mut strings = [StringState::Muted; STRINGS];
                for (i, f) in fingering.iter().enumerate() {
                    match f {
                        c if c == "x" => {},
                        c if c == "0" => strings[i] = StringState::Open,
                        c => {
                            let fret_num = c.parse::<u8>().unwrap();
                            strings[i] = StringState::FrettedOn(fret_num);
                        }
                    }
                }
                
                let fing = Fingering::new(strings, Some(chord)).unwrap();
                song_library::add_fingering(&fing).expect("Error during saving a fingering!");
            },
            Command::Show { path, key, chords, rhythm, fingerings, colored } => {
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

                song_library::show(&path, key, chords, rhythm, fingerings, colored)
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
            Command::Add(subcommand) => match subcommand {
                AddSubcommand::FromTxt { path, title, artist } => {
                    let song = Song::from_txt(
                        &path,
                        Metadata { title, artist, key: None }
                        ).expect("Error during adding a song!");

                    song_library::add(&song)
                        .expect("Error during adding a song!");
                },
                AddSubcommand::Empty { title, artist } => {
                    let song = Song::new(&title, &artist);
                    song_library::add(&song)
                        .expect("Error during adding a song!");
                }
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
