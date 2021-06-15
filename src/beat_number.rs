pub struct BeatNumber {
    pub quarter_note: u32,
    pub sixteenth_note: u32,
}

impl BeatNumber {
    pub fn new(quarter_note: u32, sixteenth_note: u32) -> Self {
        Self {
            quarter_note,
            sixteenth_note,
        }
    }
}
