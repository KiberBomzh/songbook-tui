#[cfg(feature = "tui")]
mod tui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use songbook::{Song, Metadata};


#[derive(Parser, Debug)]
#[command(name = "songbook")]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    #[command(about = "Print fretboard, for non-standart tuning use '-t'")]
    Fret {
        #[arg(short, long, default_value = "e, b, g, d, a, e", value_name = "TUNING")]
        tuning: String
    },

    #[command(about = "Print chord's fingerings")]
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
            Command::Fret {tuning} => println!("{}", tuning),
            Command::Chord {chord} => {
                if let Some(chord) = songbook::Chord::new(&chord) {
                    dbg!(&chord);
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
