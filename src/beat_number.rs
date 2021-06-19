pub struct BeatNumber {
    pub quarter_note: u32,
    pub sixteenth_note: u32,
}

impl BeatNumber {
    pub fn is_beginning_of_measure(&self) -> bool {
        self.quarter_note == 1 && self.sixteenth_note == 1
    }
}
