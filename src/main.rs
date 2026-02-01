#[cfg(feature = "tui")]
mod tui;

use std::path::PathBuf;

use clap::Parser;
use songbook::{Song, Metadata};
use songbook::Note::*;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,

    #[arg(short, long)]
    transpose: Option<i32>,
}


fn main() {
    let args = Args::parse();

    let mut song = Song::from_txt(
        &args.path,
        Metadata { title: String::new(), artist: String::new(), key: String::new() }
        ).unwrap();
    if let Some(t) = args.transpose {
        song.transpose(t)
    }

    let fings = songbook::get_chords(
        &[E, B, G, D, A, E],
        &vec!(C, E, G)
    );
    for fing in &fings {
        println!("{}\n\n", fing.get_text());
    }
    songbook::chord_fingerings::sum_text_in_fingerings(&fings);

    // dbg!(&song);
    // println!("{}", song.get_text());
}
