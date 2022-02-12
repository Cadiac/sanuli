use crate::game::{KnownCounts, KnownStates};
use crate::manager::{CharacterCount, CharacterState, TileState};

pub fn keyboard_tilestate(
    key: &char,
    current_guess: usize,
    states: &KnownStates,
    counts: &KnownCounts,
) -> TileState {
    let is_correct = states[current_guess]
        .iter()
        .any(|((c, _index), state)| c == key && state == &CharacterState::Correct);
    if is_correct {
        return TileState::Correct;
    }

    match counts[current_guess].get(key) {
        Some(CharacterCount::AtLeast(count)) => {
            if *count == 0 {
                return TileState::Unknown;
            }
            TileState::Present
        }
        Some(CharacterCount::Exactly(count)) => {
            if *count == 0 {
                return TileState::Absent;
            }
            TileState::Present
        }
        None => TileState::Unknown,
    }
}

pub fn updated_known_count(
    character: &char,
    current_guess: usize,
    guess: &[(char, TileState)],
    counts: &KnownCounts,
    word: &[char],
) -> Option<CharacterCount> {
    let known_count = counts[current_guess]
        .get(character)
        .unwrap_or(&CharacterCount::AtLeast(0));

    // At most the same amount of characters are highlighted as there are in the word
    let count_in_word = word.iter().filter(|c| *c == character).count();
    if count_in_word == 0 {
        return Some(CharacterCount::Exactly(0));
    }

    let count_in_guess = guess.iter().filter(|(c, _)| c == character).count();

    // Exact count should never change
    if let CharacterCount::AtLeast(count) = known_count {
        if count_in_guess > count_in_word {
            if count_in_word >= *count {
                // The guess had more copies of the character than the word,
                // the exact count is revealed
                return Some(CharacterCount::Exactly(count_in_word));
            }
        } else if count_in_guess == count_in_word || count_in_guess > *count {
            // One of:
            // 1) The count had the exact count but that isn't revealed yet
            // 2) Found more than before, but the exact count is still unknown
            return Some(CharacterCount::AtLeast(count_in_guess));
        }
    };

    None
}
