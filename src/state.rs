use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::mem;
use std::str::FromStr;

use chrono::{Local, NaiveDate};
use wasm_bindgen::JsValue;
use web_sys::{window, Window};

const FULL_WORDS: &str = include_str!("../full-words.txt");
const COMMON_WORDS: &str = include_str!("../common-words.txt");
const DAILY_WORDS: &str = include_str!("../daily-words.txt");
const PROFANITIES: &str = include_str!("../profanities.txt");
const EMPTY: char = '\u{00a0}'; // &nbsp;
const SUCCESS_EMOJIS: [&str; 8] = ["🥳", "🤩", "🤗", "🎉", "😊", "😺", "😎", "👏"];
pub const DEFAULT_WORD_LENGTH: usize = 5;
pub const DEFAULT_MAX_GUESSES: usize = 6;

fn parse_all_words() -> HashMap<(WordList, usize), HashSet<Vec<char>>> {
    let mut words = HashMap::with_capacity(2);
    for word in FULL_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        words
            .entry((WordList::Full, word_length))
            .or_insert(HashSet::new())
            .insert(chars.collect());
    }

    for word in COMMON_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        words
            .entry((WordList::Common, word_length))
            .or_insert(HashSet::new())
            .insert(chars.collect());
    }

    for word in PROFANITIES.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        words
            .entry((WordList::Profanities, word_length))
            .or_insert(HashSet::new())
            .insert(chars.collect());
    }

    words
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum WordList {
    Full,
    Common,
    Profanities,
}

impl FromStr for WordList {
    type Err = ();

    fn from_str(input: &str) -> Result<WordList, Self::Err> {
        match input {
            "full" => Ok(WordList::Full),
            "common" => Ok(WordList::Common),
            "profanities" => Ok(WordList::Profanities),
            _ => Err(()),
        }
    }
}

impl fmt::Display for WordList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WordList::Full => write!(f, "full"),
            WordList::Common => write!(f, "common"),
            WordList::Profanities => write!(f, "profanities"),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum GameMode {
    Classic,
    Relay,
    DailyWord,
}

impl FromStr for GameMode {
    type Err = ();

    fn from_str(input: &str) -> Result<GameMode, Self::Err> {
        match input {
            "classic" => Ok(GameMode::Classic),
            "relay" => Ok(GameMode::Relay),
            "daily_word" => Ok(GameMode::DailyWord),
            _ => Err(()),
        }
    }
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameMode::Classic => write!(f, "classic"),
            GameMode::Relay => write!(f, "relay"),
            GameMode::DailyWord => write!(f, "daily_word"),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Theme {
    Dark,
    Colorblind,
}

impl FromStr for Theme {
    type Err = ();

    fn from_str(input: &str) -> Result<Theme, Self::Err> {
        match input {
            "dark" => Ok(Theme::Dark),
            "colorblind" => Ok(Theme::Colorblind),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Theme::Dark => write!(f, "dark"),
            Theme::Colorblind => write!(f, "colorblind"),
        }
    }
}


#[derive(Clone, PartialEq)]
pub enum CharacterState {
    Correct,
    Absent,
    Unknown,
}

#[derive(Clone, PartialEq)]
pub enum TileState {
    Correct,
    Absent,
    Present,
    Unknown,
}

impl fmt::Display for TileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileState::Correct => write!(f, "correct"),
            TileState::Absent => write!(f, "absent"),
            TileState::Present => write!(f, "present"),
            TileState::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DailyWordHistory {
    date: NaiveDate,
    word: String,
    guesses: Vec<Vec<char>>,
    current_guess: usize,
    is_guessing: bool,
    is_winner: bool,
}

#[derive(Clone, PartialEq)]
pub enum CharacterCount {
    AtLeast(usize),
    Exactly(usize),
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub word: Vec<char>,

    pub current_word_list: WordList,
    pub word_lists: HashMap<(WordList, usize), HashSet<Vec<char>>>,
    pub allow_profanities: bool,

    pub word_length: usize,
    pub max_guesses: usize,

    pub is_guessing: bool,
    pub is_winner: bool,
    pub is_unknown: bool,
    pub is_reset: bool,

    pub theme: Theme,

    pub daily_word_history: HashMap<NaiveDate, DailyWordHistory>,

    pub game_mode: GameMode,
    pub previous_game_mode: GameMode,

    pub message: String,

    pub known_states: Vec<HashMap<(char, usize), CharacterState>>,
    pub discovered_counts: Vec<HashMap<char, CharacterCount>>,

    pub guesses: Vec<Vec<(char, TileState)>>,
    pub previous_guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,

    pub streak: usize,
    pub max_streak: usize,
    pub total_played: usize,
    pub total_solved: usize,
}

impl State {
    pub fn new() -> Self {
        let word_length = DEFAULT_WORD_LENGTH;
        let max_guesses = DEFAULT_MAX_GUESSES;

        let word_lists = parse_all_words();
        let current_word_list = WordList::Common;
        let allow_profanities = true;

        let word = State::get_random_word(
            &word_lists,
            current_word_list,
            word_length,
            allow_profanities,
        );

        let guesses = std::iter::repeat(Vec::with_capacity(word_length))
            .take(max_guesses)
            .collect::<Vec<_>>();

        let known_states = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let discovered_counts = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        Self {
            word,

            word_lists,
            current_word_list,
            allow_profanities,

            word_length,
            max_guesses,

            is_guessing: true,
            is_winner: false,
            is_unknown: false,
            is_reset: false,

            theme: Theme::Dark,

            daily_word_history: HashMap::new(),

            game_mode: GameMode::Classic,
            previous_game_mode: GameMode::Classic,

            message: EMPTY.to_string(),

            known_states,
            discovered_counts,

            guesses,
            previous_guesses: Vec::new(),
            current_guess: 0,
            streak: 0,
            max_streak: 0,
            total_played: 0,
            total_solved: 0,
        }
    }

    pub fn keyboard_tilestate(&self, key: &char) -> TileState {
        let is_correct = self.known_states[self.current_guess]
            .iter()
            .any(|((c, _index), state)| c == key && state == &CharacterState::Correct);
        if is_correct {
            return TileState::Correct;
        }

        match self.discovered_counts[self.current_guess].get(key) {
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

    fn current_guess_state(&mut self, character: char, index: usize) -> TileState {
        match self.known_states[self.current_guess].get(&(character, index)) {
            Some(CharacterState::Correct) => TileState::Correct,
            Some(CharacterState::Absent) => TileState::Absent,
            _ => {
                match self.discovered_counts[self.current_guess].get(&character) {
                    Some(CharacterCount::Exactly(count)) => {
                        // We may know the exact count, but not the exact index of any characters..
                        if *count == 0 {
                            return TileState::Absent;
                        }

                        let is_every_correct_found = self.known_states[self.current_guess]
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

    fn reveal_row_tiles(&mut self, row: usize) {
        if let Some(guess) = self.guesses.get_mut(row) {
            let mut revealed_count_on_row: HashMap<char, usize> =
                HashMap::with_capacity(self.word_length);

            for (index, (character, _)) in guess.iter().enumerate() {
                if let Some(CharacterState::Correct) =
                    self.known_states[row].get(&(*character, index))
                {
                    revealed_count_on_row
                        .entry(*character)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }

            for (index, (character, tile_state)) in guess.into_iter().enumerate() {
                match self.known_states[row].get(&(*character, index)) {
                    Some(CharacterState::Correct) => {
                        *tile_state = TileState::Correct;
                    }
                    Some(CharacterState::Absent) => {
                        let revealed = revealed_count_on_row
                            .entry(*character)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);

                        let discovered_count = self.discovered_counts[row]
                            .get(character)
                            .unwrap_or(&CharacterCount::AtLeast(0));

                        match discovered_count {
                            CharacterCount::AtLeast(count) | CharacterCount::Exactly(count) => {
                                if *revealed <= *count {
                                    *tile_state = TileState::Present;
                                } else {
                                    *tile_state = TileState::Absent;
                                }
                            }
                        }
                    }
                    _ => {
                        *tile_state = TileState::Unknown;
                    }
                }
            }
        }
    }

    pub fn submit_current_guess(&mut self) {
        for (index, (character, _)) in self.guesses[self.current_guess].iter().enumerate() {
            let known = self.known_states[self.current_guess]
                .entry((*character, index))
                .or_insert(CharacterState::Unknown);

            if self.word[index] == *character {
                *known = CharacterState::Correct;
            } else {
                *known = CharacterState::Absent;

                let discovered_count = self.discovered_counts[self.current_guess]
                    .entry(*character)
                    .or_insert(CharacterCount::AtLeast(0));

                // At most the same amount of characters are highlighted as there are in the word
                let count_in_word = self.word.iter().filter(|c| *c == character).count();
                if count_in_word == 0 {
                    *discovered_count = CharacterCount::Exactly(0);
                    continue;
                }

                let count_in_guess = self.guesses[self.current_guess]
                    .iter()
                    .filter(|(c, _)| c == character)
                    .count();

                match discovered_count {
                    CharacterCount::AtLeast(count) => {
                        if count_in_guess > count_in_word {
                            if count_in_word >= *count {
                                // The guess had more copies of the character than the word,
                                // the exact count is revealed
                                *discovered_count = CharacterCount::Exactly(count_in_word);
                            }
                        } else if count_in_guess == count_in_word {
                            // The count had the exact count but that isn't revealed yet
                            *discovered_count = CharacterCount::AtLeast(count_in_guess);
                        } else if count_in_guess > *count {
                            // Found more, but the exact count is still unknown
                            *discovered_count = CharacterCount::AtLeast(count_in_guess);
                        }
                    }
                    // Exact count should never change
                    CharacterCount::Exactly(_) => {}
                }
            }
        }

        // Copy the previous knowledge to the next round
        if self.current_guess < self.max_guesses - 1 {
            let next = self.current_guess + 1;
            self.known_states[next] = self.known_states[self.current_guess].clone();
            self.discovered_counts[next] = self.discovered_counts[self.current_guess].clone();
        }

        self.reveal_row_tiles(self.current_guess);
    }

    pub fn push_character(&mut self, character: char) -> bool {
        if !self.is_guessing || self.guesses[self.current_guess].len() >= self.word_length {
            return false;
        }

        self.clear_message();

        let tile_state =
            self.current_guess_state(character, self.guesses[self.current_guess].len());
        self.guesses[self.current_guess].push((character, tile_state));
        true
    }

    pub fn pop_character(&mut self) -> bool {
        if !self.is_guessing || self.guesses[self.current_guess].is_empty() {
            return false;
        }

        self.clear_message();
        self.guesses[self.current_guess].pop();

        true
    }

    fn is_guess_allowed(&self) -> bool {
        self.is_guessing && self.guesses[self.current_guess].len() == self.word_length
    }

    fn is_guess_real_word(&self) -> bool {
        match self.word_lists.get(&(WordList::Full, self.word_length)) {
            Some(list) => {
                let word: &Vec<char> = &self.guesses[self.current_guess]
                    .iter()
                    .map(|(c, _)| *c)
                    .collect();

                return list.contains(word);
            }
            None => false,
        }
    }

    fn is_correct_word(&self) -> bool {
        self.guesses[self.current_guess]
            .iter()
            .map(|(c, _)| *c)
            .collect::<Vec<char>>()
            == self.word
    }

    fn is_game_ended(&self) -> bool {
        self.is_winner || self.current_guess == self.max_guesses - 1
    }

    fn clear_message(&mut self) {
        self.is_unknown = false;
        self.message = EMPTY.to_string();
    }

    fn set_game_end_message(&mut self) {
        if self.is_winner {
            if self.game_mode == GameMode::DailyWord {
                self.message = format!(
                    "Löysit päivän sanulin! {}",
                    SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap()
                );
            } else {
                self.message = format!(
                    "Löysit sanan! {}",
                    SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap()
                );
            }
        } else {
            self.message = format!("Sana oli \"{}\"", self.word.iter().collect::<String>());
        }
    }

    fn set_daily_word_history(&mut self, date: &NaiveDate) {
        self.daily_word_history.insert(
            *date,
            DailyWordHistory {
                word: self.word.iter().collect(),
                date: *date,
                guesses: self
                    .guesses
                    .iter()
                    .map(|guess| guess.iter().map(|(c, _)| *c).collect())
                    .collect(),
                current_guess: self.current_guess,
                is_guessing: self.is_guessing,
                is_winner: self.is_winner,
            },
        );
    }

    fn set_game_statistics(&mut self) {
        self.total_played += 1;

        if self.is_winner {
            self.total_solved += 1;

            if self.game_mode != GameMode::DailyWord {
                self.streak += 1;
                if self.streak > self.max_streak {
                    self.max_streak = self.streak;
                }
            }
        } else {
            self.streak = 0;
        }
    }

    pub fn submit_guess(&mut self) -> bool {
        if !self.is_guess_allowed() {
            self.message = "Liian vähän kirjaimia!".to_owned();
            return true;
        }
        if !self.is_guess_real_word() {
            self.is_unknown = true;
            self.message = "Ei sanulistalla.".to_owned();
            return true;
        }

        self.is_reset = false;
        self.clear_message();

        self.is_winner = self.is_correct_word();
        self.submit_current_guess();
        if self.is_game_ended() {
            self.is_guessing = false;

            self.set_game_statistics();
            self.set_game_end_message();

            let _result = self.persist_stats();
        } else {
            self.current_guess += 1;
        }

        if self.game_mode == GameMode::DailyWord {
            let today = Local::now().naive_local().date();
            self.set_daily_word_history(&today);

            let _result = self.persist_single_daily_word(&today);
        } else {
            let _result = self.persist_game();
        }

        true
    }

    pub fn get_random_word(
        word_lists: &HashMap<(WordList, usize), HashSet<Vec<char>>>,
        current: WordList,
        word_length: usize,
        allow_profanities: bool,
    ) -> Vec<char> {
        let mut words = word_lists
            .get(&(current, word_length))
            .unwrap()
            .iter()
            .collect::<Vec<_>>();

        if !allow_profanities {
            if let Some(profanities) = word_lists.get(&(WordList::Profanities, word_length)) {
                words.retain(|word| !profanities.contains(*word));
            }
        }

        let chosen = words.choose(&mut rand::thread_rng()).unwrap();
        (*chosen).clone()
    }

    pub fn get_daily_word_index(&self) -> usize {
        let epoch = NaiveDate::from_ymd(2022, 1, 07); // Epoch of the daily word mode, index 0
        Local::now()
            .naive_local()
            .date()
            .signed_duration_since(epoch)
            .num_days() as usize
    }

    pub fn get_daily_word(&self) -> Vec<char> {
        DAILY_WORDS
            .lines()
            .nth(self.get_daily_word_index())
            .unwrap()
            .chars()
            .collect()
    }

    pub fn create_new_game(&mut self) -> bool {
        let next_word = if self.game_mode == GameMode::DailyWord {
            self.get_daily_word()
        } else {
            State::get_random_word(
                &self.word_lists,
                self.current_word_list,
                self.word_length,
                self.allow_profanities,
            )
        };

        let previous_word = mem::replace(&mut self.word, next_word);

        if previous_word.len() <= self.word_length {
            self.previous_guesses = mem::take(&mut self.guesses);
            self.previous_guesses.truncate(self.current_guess);
        } else {
            let previous_guesses = mem::take(&mut self.guesses);
            self.previous_guesses = previous_guesses
                .into_iter()
                .map(|guess| guess.into_iter().take(self.word_length).collect())
                .collect();
            self.previous_guesses.truncate(self.current_guess);
        }

        self.guesses = Vec::with_capacity(self.max_guesses);

        self.known_states = std::iter::repeat(HashMap::new())
            .take(DEFAULT_MAX_GUESSES)
            .collect::<Vec<_>>();
        self.discovered_counts = std::iter::repeat(HashMap::new())
            .take(DEFAULT_MAX_GUESSES)
            .collect::<Vec<_>>();

        if previous_word.len() == self.word_length
            && self.is_winner
            && self.game_mode == GameMode::Relay
        {
            let empty_guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
                .take(self.max_guesses - 1)
                .collect::<Vec<_>>();

            self.guesses.push(
                previous_word
                    .iter()
                    .map(|c| (*c, TileState::Unknown))
                    .collect(),
            );
            self.guesses.extend(empty_guesses);

            self.current_guess = 0;
            self.submit_current_guess();
            self.current_guess = 1;
        } else {
            self.guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
                .take(self.max_guesses)
                .collect::<Vec<_>>();
            self.current_guess = 0;
        }

        self.is_guessing = true;
        self.is_winner = false;
        self.is_reset = true;
        self.clear_message();

        if self.game_mode == GameMode::DailyWord {
            let today = Local::now().naive_local().date();
            if let Some(solve) = self.daily_word_history.get(&today).cloned() {
                for (guess_index, guess) in solve.guesses.iter().enumerate() {
                    self.guesses[guess_index] =
                        guess.iter().map(|c| (*c, TileState::Unknown)).collect();
                    self.current_guess = guess_index;
                    self.submit_current_guess();
                }
                self.is_winner = solve.is_winner;
                self.is_guessing = solve.is_guessing;
                self.current_guess = solve.current_guess;
            }

            if !self.is_guessing {
                self.message = "Uusi sanuli huomenna!".to_owned();
            }
        } else {
            let _result = self.persist_game();
        }

        true
    }

    pub fn change_word_length(&mut self, new_length: usize) {
        self.word_length = new_length;

        // TODO: Store streaks for every word length separately

        if self.game_mode == GameMode::DailyWord {
            self.game_mode = GameMode::Classic;
        }
    }

    pub fn change_game_mode(&mut self, new_mode: GameMode) {
        self.previous_game_mode = std::mem::replace(&mut self.game_mode, new_mode);
        self.message = EMPTY.to_string();
        let _result = self.persist_settings();

        if self.game_mode == GameMode::DailyWord {
            self.word_length = 5;
        }
    }

    pub fn change_word_list(&mut self, new_list: WordList) {
        self.current_word_list = new_list;
        self.message = EMPTY.to_string();
        let _result = self.persist_settings();
    }

    pub fn change_allow_profanities(&mut self, is_allowed: bool) {
        self.allow_profanities = is_allowed;
        let _result = self.persist_settings();
    }

    pub fn change_theme(&mut self, theme: Theme) -> bool {
        self.theme = theme;
        let _result = self.persist_settings();
        true
    }

    // Persisting & restoring game state

    fn persist_settings(&mut self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        let local_storage = window.local_storage().expect("local storage not available");
        if let Some(local_storage) = local_storage {
            local_storage.set_item("game_mode", &self.game_mode.to_string())?;
            local_storage.set_item("word_length", format!("{}", self.word_length).as_str())?;
            local_storage.set_item("word_list", format!("{}", self.current_word_list).as_str())?;
            local_storage.set_item("allow_profanities", format!("{}", self.allow_profanities).as_str())?;
            local_storage.set_item("theme", format!("{}", self.theme).as_str())?;
        }

        Ok(())
    }

    fn persist_stats(&self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        let local_storage = window.local_storage().expect("local storage not available");
        if let Some(local_storage) = local_storage {
            local_storage.set_item("streak", &format!("{}", self.streak))?;
            local_storage.set_item("max_streak", &format!("{}", self.max_streak))?;
            local_storage.set_item("total_played", &format!("{}", self.total_played))?;
            local_storage.set_item("total_solved", &format!("{}", self.total_solved))?;
        }

        Ok(())
    }

    fn persist_game(&self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        let local_storage = window.local_storage().expect("local storage not available");
        if let Some(local_storage) = local_storage {
            local_storage.set_item("word", &self.word.iter().collect::<String>())?;
            local_storage.set_item("word_length", &format!("{}", self.word_length))?;
            local_storage.set_item("current_guess", &format!("{}", self.current_guess))?;
            local_storage.set_item(
                "guesses",
                &self
                    .guesses
                    .iter()
                    .map(|guess| guess.iter().map(|(c, _)| c).collect::<String>())
                    .collect::<Vec<String>>()
                    .join(","),
            )?;
            local_storage.set_item("message", &self.message)?;
            local_storage.set_item("is_guessing", format!("{}", self.is_guessing).as_str())?;
            local_storage.set_item("is_winner", format!("{}", self.is_winner).as_str())?;
        }

        Ok(())
    }

    fn persist_single_daily_word(&self, date: &NaiveDate) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        let local_storage = window.local_storage().expect("local storage not available");

        if let Some(local_storage) = local_storage {
            if let Some(history) = self.daily_word_history.get(date) {
                local_storage.set_item(
                    &format!("daily_word_history[{}]", date.format("%Y-%m-%d")),
                    &format!(
                        "{}|{}|{}|{}|{}|{}",
                        history.word,
                        history.date.format("%Y-%m-%d"),
                        history
                            .guesses
                            .iter()
                            .map(|guess| guess.iter().collect::<String>())
                            .collect::<Vec<_>>()
                            .join(","),
                        history.current_guess,
                        history.is_guessing,
                        history.is_winner
                    ),
                )?;
            }

            local_storage.set_item(
                "daily_word_history",
                &format!(
                    "{}",
                    &self
                        .daily_word_history
                        .keys()
                        .map(|date| date.format("%Y-%m-%d").to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )?;

            local_storage.set_item("total_played", &format!("{}", self.total_played))?;
            local_storage.set_item("total_solved", &format!("{}", self.total_solved))?;
        }

        Ok(())
    }

    fn rehydrate_daily_word(&mut self) {
        self.word = self.get_daily_word();
        self.word_length = self.word.len();

        let today = Local::now().naive_local().date();
        if let Some(solve) = self.daily_word_history.get(&today).cloned() {
            for (guess_index, guess) in solve.guesses.iter().enumerate() {
                self.guesses[guess_index] =
                    guess.iter().map(|c| (*c, TileState::Unknown)).collect();
                self.current_guess = guess_index;
                self.submit_current_guess();
            }
            self.is_guessing = solve.is_guessing;
            self.is_winner = solve.is_winner;
            self.current_guess = solve.current_guess;

            if !self.is_guessing {
                self.message = "Uusi sanuli huomenna!".to_owned();
            } else {
                self.message = EMPTY.to_string()
            }
        }
    }

    fn rehydrate_game(&mut self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        if let Some(local_storage) = window.local_storage()? {
            if let Some(word_list_str) = local_storage.get_item("word_list")? {
                if let Ok(word_list) = word_list_str.parse::<WordList>() {
                    self.current_word_list = word_list;
                }
            }

            if let Some(word_length_str) = local_storage.get_item("word_length")? {
                if let Ok(word_length) = word_length_str.parse::<usize>() {
                    self.word_length = word_length;
                }
            }

            if let Some(allow_profanities_str) = local_storage.get_item("allow_profanities")? {
                if let Ok(allow_profanities) = allow_profanities_str.parse::<bool>() {
                    self.allow_profanities = allow_profanities;
                }
            }

            if let Some(theme_str) = local_storage.get_item("theme")? {
                if let Ok(theme) = theme_str.parse::<Theme>() {
                    self.theme = theme;
                }
            }

            if let Some(word) = local_storage.get_item("word")? {
                self.word = word.chars().collect();
            } else {
                local_storage.set_item("word", &self.word.iter().collect::<String>())?;
            }

            if let Some(is_guessing_str) = local_storage.get_item("is_guessing")? {
                if let Ok(is_guessing) = is_guessing_str.parse::<bool>() {
                    self.is_guessing = is_guessing;
                }
            }

            if let Some(is_winner_str) = local_storage.get_item("is_winner")? {
                if let Ok(is_winner) = is_winner_str.parse::<bool>() {
                    self.is_winner = is_winner;
                }
            }

            if let Some(guesses_str) = local_storage.get_item("guesses")? {
                let previous_guesses = guesses_str
                    .split(',')
                    .map(|guess| guess.chars().map(|c| (c, TileState::Unknown)).collect());

                for (guess_index, guess) in previous_guesses.enumerate() {
                    self.guesses[guess_index] = guess;
                    self.current_guess = guess_index;
                    self.submit_current_guess();
                }
            }

            if let Some(current_guess_str) = local_storage.get_item("current_guess")? {
                if let Ok(current_guess) = current_guess_str.parse::<usize>() {
                    self.current_guess = current_guess;
                }
            }
        }

        Ok(())
    }

    pub fn rehydrate(&mut self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        if let Some(local_storage) = window.local_storage().expect("local storage not available") {
            // Common state
            if let Some(game_mode_str) = local_storage.get_item("game_mode")? {
                if let Ok(new_mode) = game_mode_str.parse::<GameMode>() {
                    self.previous_game_mode = mem::replace(&mut self.game_mode, new_mode);
                }
            }

            if let Some(daily_word_history_str) = local_storage.get_item("daily_word_history")? {
                if daily_word_history_str.len() != 0 {
                    daily_word_history_str.split(',').for_each(|date_str| {
                        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
                        let daily_item = local_storage
                            .get_item(&format!("daily_word_history[{}]", date_str))
                            .unwrap();
                        if let Some(daily_str) = daily_item {
                            let parts = daily_str.split('|').collect::<Vec<&str>>();

                            // AIVAN|2022-01-07|KOIRA,AVAIN,AIVAN,,,|2|true|true
                            let word = parts[0];
                            let guesses = parts[2]
                                .split(',')
                                .map(|guess| guess.chars().collect::<Vec<_>>())
                                .collect::<Vec<_>>();
                            let current_guess = parts[3].parse::<usize>().unwrap();
                            let is_guessing = parts[4].parse::<bool>().unwrap();
                            let is_winner = parts[5].parse::<bool>().unwrap();

                            let history = DailyWordHistory {
                                word: word.to_string(),
                                date,
                                guesses: guesses,
                                current_guess: current_guess,
                                is_guessing: is_guessing,
                                is_winner: is_winner,
                            };

                            self.daily_word_history.insert(date, history);
                        }
                    });
                }
            }

            if let Some(message_str) = local_storage.get_item("message")? {
                self.message = message_str;
            }

            // Stats
            if let Some(streak_str) = local_storage.get_item("streak")? {
                if let Ok(streak) = streak_str.parse::<usize>() {
                    self.streak = streak;
                }
            }

            if let Some(max_streak_str) = local_storage.get_item("max_streak")? {
                if let Ok(max_streak) = max_streak_str.parse::<usize>() {
                    self.max_streak = max_streak;
                }
            }

            if let Some(total_played_str) = local_storage.get_item("total_played")? {
                if let Ok(total_played) = total_played_str.parse::<usize>() {
                    self.total_played = total_played;
                }
            }

            if let Some(total_solved_str) = local_storage.get_item("total_solved")? {
                if let Ok(total_solved) = total_solved_str.parse::<usize>() {
                    self.total_solved = total_solved;
                }
            }

            // Gamemode specific
            match self.game_mode {
                GameMode::DailyWord => {
                    self.rehydrate_daily_word();
                }
                GameMode::Classic | GameMode::Relay => {
                    self.rehydrate_game()?;
                }
            }
        }

        Ok(())
    }
}
