#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct BeatNumber {
    pub sixteenth_note: u32,
}

impl BeatNumber {
    pub fn is_beginning_of_measure(&self) -> bool {
        self.sixteenth_note == 0
    }

    pub fn is_next_beginning_of_measure(&self) -> bool {
        self.sixteenth_note == 15
    }

    pub fn minus_sixteenths(&self, num_sixteenths: u32) -> BeatNumber {
        BeatNumber {
            sixteenth_note: (self.sixteenth_note as i32 - num_sixteenths as i32) as u32 % 16,
        }
    }

    pub fn add_sixteenths(&self, num_sixteenths: u32) -> BeatNumber {
        BeatNumber {
            sixteenth_note: (self.sixteenth_note + num_sixteenths) % 16,
        }
    }

    pub fn duration_since(&self, other: &BeatNumber) -> u32 {
        (self.sixteenth_note + 16 - other.sixteenth_note) % 16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minus_sixteenths_same_measure() {
        assert_eq!(
            BeatNumber { sixteenth_note: 2 }.minus_sixteenths(2),
            BeatNumber { sixteenth_note: 0 }
        );
    }

    #[test]
    fn minus_sixteenths_wrap_around_measure() {
        assert_eq!(
            BeatNumber { sixteenth_note: 2 }.minus_sixteenths(3),
            BeatNumber { sixteenth_note: 15 }
        );
    }

    #[test]
    fn add_sixteenths_same_measure() {
        assert_eq!(
            BeatNumber { sixteenth_note: 1 }.add_sixteenths(2),
            BeatNumber { sixteenth_note: 3 }
        );
    }

    #[test]
    fn add_sixteenths_wrap_around_measure() {
        assert_eq!(
            BeatNumber { sixteenth_note: 14 }.add_sixteenths(2),
            BeatNumber { sixteenth_note: 0 }
        );
    }

    #[test]
    fn duration_since_simple() {
        assert_eq!(
            BeatNumber { sixteenth_note: 14 }.duration_since(&BeatNumber { sixteenth_note: 2 }),
            12
        );
    }

    #[test]
    fn duration_since_wrap_around() {
        assert_eq!(
            BeatNumber { sixteenth_note: 2 }.duration_since(&BeatNumber { sixteenth_note: 15 }),
            3
        );
    }
}
