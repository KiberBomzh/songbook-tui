#[cfg(feature = "tui")]
mod tui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use songbook::{Song, Metadata, Note, STRINGS};


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
    },

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
    Rm { path: PathBuf },

    /// Move(or rename) a song or a dir
    Mv { input_path: PathBuf, output_path: PathBuf },

    /// Print songs from the library
    Ls { path: Option<PathBuf> },

    /// Create directory in the library
    Mkdir { path: PathBuf },
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
            Command::Show { path } => {
                songbook::song_library::show(&path)
                    .expect("Error during geting song!");
            },
            Command::Add {path, artist, title} => {
                let song = Song::from_txt(
                    &path,
                    Metadata { title, artist }
                    ).expect("Error during adding a song!");

                songbook::song_library::add(&song)
                    .expect("Error during adding a song!");
            },
            Command::Rm { path } => {
                songbook::song_library::rm(&path)
                    .expect("Error during removing!");
            },
            Command::Mv {input_path, output_path } => {
                songbook::song_library::mv(&input_path, &output_path)
                    .expect("Error during moving!");
            },
            Command::Ls { path } => {
                songbook::song_library::ls(path.as_deref())
                    .expect("Error during reading a dir: ");
            },
            Command::Mkdir { path } => {
                songbook::song_library::mkdir(&path)
                    .expect("Error during creating a dir!");
            },
        }
    }
}
