use std::collections::HashMap;

use crate::game::{KnownCounts, KnownStates};
use crate::manager::{CharacterCount, CharacterState, TileState};

pub fn known_count(
    character: &char,
    current_guess: usize,
    guess: &[(char, TileState)],
    counts: &[KnownCounts],
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

fn revealed_by_char(
    guess: &[(char, TileState)],
    current_guess: usize,
    states: &[KnownStates],
) -> HashMap<char, usize> {
    let mut revealed_count_on_row: HashMap<char, usize> = HashMap::with_capacity(guess.len());

    for (index, (character, _)) in guess.iter().enumerate() {
        if let Some(CharacterState::Correct) = states[current_guess].get(&(*character, index)) {
            revealed_count_on_row
                .entry(*character)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    revealed_count_on_row
}

pub fn update_guess_tile_states(
    guess: &mut [(char, TileState)],
    guess_index: usize,
    states: &[KnownStates],
    counts: &[KnownCounts],
) {
    let mut revealed_counts = revealed_by_char(guess, guess_index, states);

    for (index, (character, tile_state)) in guess.iter_mut().enumerate() {
        *tile_state = board_tile_state(
            &mut revealed_counts,
            guess_index,
            states,
            counts,
            index,
            character,
        );
    }
}

pub fn board_tile_state(
    revealed_counts: &mut HashMap<char, usize>,
    current_guess: usize,
    states: &[KnownStates],
    counts: &[KnownCounts],
    index: usize,
    character: &char,
) -> TileState {
    match states[current_guess].get(&(*character, index)) {
        Some(CharacterState::Correct) => {
            return TileState::Correct;
        }
        Some(CharacterState::Absent) => {
            let revealed = revealed_counts
                .entry(*character)
                .and_modify(|count| *count += 1)
                .or_insert(1);

            let discovered_count = counts[current_guess]
                .get(character)
                .unwrap_or(&CharacterCount::AtLeast(0));

            match discovered_count {
                CharacterCount::AtLeast(count) | CharacterCount::Exactly(count) => {
                    if *revealed <= *count {
                        return TileState::Present;
                    } else {
                        return TileState::Absent;
                    }
                }
            }
        }
        _ => {
            return TileState::Unknown;
        }
    }
}

pub fn hint_tile_state(
    character: char,
    character_index: usize,
    guess_index: usize,
    states: &[KnownStates],
    counts: &[KnownCounts],
) -> TileState {
    match states[guess_index].get(&(character, character_index)) {
        Some(CharacterState::Correct) => TileState::Correct,
        Some(CharacterState::Absent) => TileState::Absent,
        _ => {
            match counts[guess_index].get(&character) {
                Some(CharacterCount::Exactly(count)) => {
                    // We may know the exact count, but not the exact index of any characters..
                    if *count == 0 {
                        return TileState::Absent;
                    }

                    let is_every_correct_found = states[guess_index]
                        .iter()
                        .filter(|((c, _i), state)| {
                            c == &character && *state == &CharacterState::Correct
                        })
                        .count()
                        == *count;

                    if !is_every_correct_found {
                        return TileState::Present;
                    }

                    TileState::Absent
                }
                Some(CharacterCount::AtLeast(_)) => TileState::Present,
                None => TileState::Unknown,
            }
        }
    }
}

pub fn keyboard_tile_state(
    key: &char,
    current_guess: usize,
    states: &[KnownStates],
    counts: &[KnownCounts],
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

pub fn update_known_information(
    states: &mut [KnownStates],
    counts: &mut [KnownCounts],
    guess: &mut [(char, TileState)],
    guess_index: usize,
    word: &[char],
    max_guesses: usize,
) {
    for (index, (character, _)) in guess.iter().enumerate() {
        let known = states[guess_index]
            .entry((*character, index))
            .or_insert(CharacterState::Unknown);

        if word[index] == *character {
            *known = CharacterState::Correct;
        } else {
            *known = CharacterState::Absent;

            if let Some(updated_count) = known_count(character, guess_index, guess, counts, word) {
                counts[guess_index].insert(*character, updated_count);
            }
        }
    }

    // Copy the previous knowledge to the next guess
    if guess_index < max_guesses - 1 {
        let next = guess_index + 1;
        states[next] = states[guess_index].clone(); // TODO: This sometimes breaks to panicked at 'index out of bounds: the len is 6 but the index is 6', src/logic.rs:217:9
        counts[next] = counts[guess_index].clone();
    }

    update_guess_tile_states(guess, guess_index, states, counts);
}
