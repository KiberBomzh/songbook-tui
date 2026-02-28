use crate::chord_generator::{
    STRINGS,
    chord_fingerings::StringState
};
use crate::STANDART_TUNING;


// Bar - такт
// Time - размер
// Beat - доля

pub struct Tablature {
    bars: Vec<Bar>
}

struct Bar {
    time: Time,
    beats: Vec<Beat>
} // TODO добавить слайды, хаммеры, пуллоффы и т.д.

enum Beat {
    strings: [StringState; STRINGS],
    value: NoteValue
}

enum Time {
    FourFour,
    ThreeFour
} // TODO дописать остальные размеры

enum NoteValue {
    Whole,
    Half,
    Quater,
    Eighth,
    Sixteenth
}
