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

    #[command(about = "Print song from .txt file")]
    Text {
        path: PathBuf,

        #[arg(short, long, value_name = "STEPS")]
        transpose: Option<i32>,
    } 
}


fn main() {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Command::Fret {tuning} => {
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
            Command::Chord {chord} => {
                if let Some(chord) = songbook::Chord::new(&chord) {
                    let fings = chord.get_fingerings(&songbook::STANDART_TUNING);
                    if let Some(text) = songbook::sum_text_in_fingerings(&fings) {
                        println!("{text}");
                    }
                } else {
                    println!("Unknown chord!");
                }
            },
            Command::Text {path, transpose} => {
                let mut song = Song::from_txt(
                    &path,
                    Metadata { title: String::new(), artist: String::new() }
                    ).unwrap();
                if let Some(t) = transpose {
                    song.transpose(t)
                }

                song.print();
            }
        }
    }
}
