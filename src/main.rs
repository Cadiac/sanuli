extern crate serde_scan;

use rand::seq::SliceRandom;
use std::collections::HashMap;
use yew::{
    classes, function_component, html, Callback, Children,
    Component, Context, Html, MouseEvent, Properties,
};

const WORDS: &str = include_str!("../vendor/kotus-sanalista_v1/kotus-sanalista_v1.xml");

fn parse_words(input: &str) -> Vec<Vec<char>> {
    let parts = input.split("<kotus-sanalista>\n").collect::<Vec<&str>>();
    let words = parts[1].split("</kotus-sanalista>").collect::<Vec<&str>>();

    let mut word_list = Vec::new();

    for line in words[0].lines() {
        let (word, _): (String, String) = serde_scan::scan!("<st><s>{}</s>{}" <- line).unwrap();

        if word.len() == 5 {
            word_list.push(word.to_uppercase().chars().collect());
        }
    }

    word_list
}

enum Msg {
    KeyPress(char),
    Backspace,
    Submit,
}

struct Model {
    word_list: Vec<Vec<char>>,
    word: Vec<char>,
    is_guessing: bool,
    is_winner: bool,
    characters: HashMap<char, CharacterState>,
    guesses: [Vec<char>; 6],
    current_guess: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CharacterState {
    Unknown,
    NotInWord,
    InWord,
    InCorrectPosition,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let word_list = parse_words(WORDS);

        let word = word_list.choose(&mut rand::thread_rng()).unwrap().clone();

        Self {
            word,
            word_list,
            is_guessing: true,
            is_winner: false,
            characters: HashMap::new(),
            guesses: [
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
            ],
            current_guess: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyPress(c) => {
                if !self.is_guessing || self.guesses[self.current_guess].len() >= 5 {
                    return false;
                }

                self.guesses[self.current_guess].push(c);

                true
            }
            Msg::Backspace => {
                if !self.is_guessing || self.guesses[self.current_guess].len() <= 0 {
                    return false;
                }

                self.guesses[self.current_guess].pop();

                true
            }
            Msg::Submit => {
                if self.guesses[self.current_guess].len() != 5 {
                    return false;
                }

                if !self.word_list.contains(&self.guesses[self.current_guess]) {
                    return false;
                }

                self.is_winner = self.guesses[self.current_guess] == self.word;

                for (index, character) in self.guesses[self.current_guess].iter().enumerate() {
                    let current_state = self
                        .characters
                        .entry(*character)
                        .or_insert(CharacterState::Unknown);

                    if *current_state == CharacterState::InCorrectPosition {
                        continue;
                    }

                    if self.word[index] == *character {
                        *current_state = CharacterState::InCorrectPosition;
                        continue;
                    }

                    if *current_state == CharacterState::InWord {
                        continue;
                    }

                    if self.word.contains(character) {
                        *current_state = CharacterState::InWord;
                    } else {
                        *current_state = CharacterState::NotInWord;
                    }
                }

                if self.current_guess == 5 || self.is_winner {
                    self.is_guessing = false;
                } else {
                    self.current_guess += 1;
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();

        let keyboard = vec![
            vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å'],
            vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'],
            vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
        ];

        html! {
            <div class="game">
                <header>
                    <div class="title">{ "Sanuli" }</div>
                    <div class="subtitle">{ "Voititko? "}{ self.is_winner }</div>
                    <div class="subtitle">{ "Sana: "}{ self.word.iter().collect::<String>() }</div>
                </header>

                <div class="board-container">
                    <div class="board">
                        { self.guesses.iter().map(|guess| html! {
                            <div class="row">
                                { (0..5).map(|i| html! {
                                    <div class={classes!("tile",
                                        guess.get(i).and_then(|c| self.characters.get(c).and_then(map_character_state)))}>
                                        { guess.get(i).unwrap_or(&' ') }
                                    </div>
                                }).collect::<Html>() }
                            </div>
                        }).collect::<Html>() }
                    </div>
                </div>

                <div>
                    <div>
                        {
                            keyboard[0].iter().cloned().map(|key| html! {
                                <button class={classes!(
                                    "keyboard-button",
                                    self.characters.get(&key).and_then(map_character_state))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                    </div>
                    <div>
                        {
                            keyboard[1].iter().cloned().map(|key| html! {
                                <button class={classes!(
                                    "keyboard-button",
                                    self.characters.get(&key).and_then(map_character_state))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                    </div>
                    <div>
                        {
                            keyboard[2].iter().cloned().map(|key| html! {
                                <button class={classes!(
                                    "keyboard-button",
                                    self.characters.get(&key).and_then(map_character_state))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                        <button class={classes!("keyboard-button")}
                            onclick={link.callback(move |_| Msg::Backspace)}>{ "<x]" }</button>
                        <button onclick={link.callback(|_| Msg::Submit)}>{ "ARVAA" }</button>
                    </div>
                </div>
            </div>
        }
    }
}

fn map_character_state(state: &CharacterState) -> Option<&'static str> {
    match state {
        CharacterState::InWord => Some("in-word"),
        CharacterState::NotInWord => Some("not-in-word"),
        CharacterState::InCorrectPosition => Some("in-correct-position"),
        CharacterState::Unknown => None,
    }
}

fn main() {
    yew::start_app::<Model>();
}
