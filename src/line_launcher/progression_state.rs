use crate::{Chord, Progression};

enum ProgressionChordIndexState {
    HaventStarted,
    AtChordIndex(usize),
}

pub struct ProgressionState<'progression> {
    progression: &'progression Progression,
    chord_index_state: ProgressionChordIndexState,
}

impl<'progression> ProgressionState<'progression> {
    pub fn new(progression: &'progression Progression) -> Self {
        Self {
            progression,
            chord_index_state: ProgressionChordIndexState::HaventStarted,
        }
    }

    pub fn chord_index(&self) -> usize {
        if let ProgressionChordIndexState::AtChordIndex(chord_index) = self.chord_index_state {
            chord_index
        } else {
            0
        }
    }

    pub fn current_chord(&self) -> &Chord {
        &self.progression.chords[self.chord_index()]
    }

    pub fn tick_measure(&mut self) {
        self.chord_index_state = match self.chord_index_state {
            ProgressionChordIndexState::HaventStarted => {
                ProgressionChordIndexState::AtChordIndex(0)
            }
            ProgressionChordIndexState::AtChordIndex(chord_index) => {
                ProgressionChordIndexState::AtChordIndex(
                    (chord_index + 1) % self.progression.chords.len(),
                )
            }
        }
    }

    pub fn has_started(&self) -> bool {
        !matches!(
            self.chord_index_state,
            ProgressionChordIndexState::HaventStarted
        )
    }
}
