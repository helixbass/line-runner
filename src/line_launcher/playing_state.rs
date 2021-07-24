#[derive(Clone, Copy, Debug)]
pub enum PlayingState {
    NotPlaying,
    Playing {
        line_index: usize,
        next_note_index: usize,
        pitch_offset: i8,
        next_note_off_index: usize,
    },
}
