#[cfg(feature = "tui")]
mod tui;

use std::path::PathBuf;

use clap::Parser;
use songbook::{Song, Metadata};


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
        Metadata { title: String::new(), artist: String::new() }
        ).unwrap();
    if let Some(t) = args.transpose {
        song.transpose(t)
    }


    let mut fings = Vec::new();
    for f in song.get_fingerings() {
        fings.push(f[0].clone());
    }

    if let Some(text) = songbook::sum_text_in_fingerings(&fings) {
        println!("{text}");
    }

    // dbg!(&song);
    println!("{}", song.get_text());
}
