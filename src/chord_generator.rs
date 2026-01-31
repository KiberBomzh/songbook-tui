use crate::chord_generator::Note::*;


#[derive(Debug, Copy, Clone)]
pub enum Note {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp
}

impl Note {
    fn do_step_right(&self) -> Self {
        match self {
            A =>      ASharp,
            ASharp => B,
            B =>      C,
            C =>      CSharp,
            CSharp => D,
            D =>      DSharp,
            DSharp => E,
            E =>      F,
            F =>      FSharp,
            FSharp => G,
            G =>      GSharp,
            GSharp => A
        }
    }
    fn do_step_left(&self) -> Self {
        match self {
            A =>      GSharp,
            ASharp => A,
            B =>      ASharp,
            C =>      B,
            CSharp => C,
            D =>      CSharp,
            DSharp => D,
            E =>      DSharp,
            F =>      E,
            FSharp => F,
            G =>      FSharp,
            GSharp => G
        }
    }
}

pub fn get_fretboard(tuning: &[Note; 6]) -> [[Note; 25]; 6] {
    let mut fretboard = [[A; 25]; 6];
    for (index, note) in tuning.iter().enumerate() {
        let mut note = note.clone();
        for n in &mut fretboard[index] {
            *n = note;
            note = note.do_step_right();
        }
    }

    return fretboard
}
