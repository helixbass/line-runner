#[derive(PartialEq, Debug)]
pub struct BeatNumber {
    pub quarter_note: u32,
    pub sixteenth_note: u32,
}

impl BeatNumber {
    pub fn is_beginning_of_measure(&self) -> bool {
        self.quarter_note == 1 && self.sixteenth_note == 1
    }

    pub fn minus_sixteenths(&self, num_sixteenths: u32) -> BeatNumber {
        if num_sixteenths < self.sixteenth_note {
            return BeatNumber {
                quarter_note: self.quarter_note,
                sixteenth_note: self.sixteenth_note - num_sixteenths,
            };
        }
        let num_quarter_notes = (num_sixteenths / 4) + 1;
        BeatNumber {
            quarter_note: (self.quarter_note as i32 - num_quarter_notes as i32 - 1) as u32 % 4 + 1,
            sixteenth_note: (self.sixteenth_note as i32 - num_sixteenths as i32 - 1) as u32 % 4 + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minus_sixteenths_same_beat() {
        assert_eq!(
            BeatNumber {
                quarter_note: 1,
                sixteenth_note: 3
            }
            .minus_sixteenths(2),
            BeatNumber {
                quarter_note: 1,
                sixteenth_note: 1
            }
        );
    }

    #[test]
    fn minus_sixteenths_previous_beat() {
        assert_eq!(
            BeatNumber {
                quarter_note: 2,
                sixteenth_note: 3
            }
            .minus_sixteenths(3),
            BeatNumber {
                quarter_note: 1,
                sixteenth_note: 4
            }
        );
    }

    #[test]
    fn minus_sixteenths_wrap_around_measure() {
        assert_eq!(
            BeatNumber {
                quarter_note: 1,
                sixteenth_note: 3
            }
            .minus_sixteenths(3),
            BeatNumber {
                quarter_note: 4,
                sixteenth_note: 4
            }
        );
    }
}
