extern crate serde_scan;

use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::collections::HashSet;
use yew::{
    classes, function_component, html, Callback, Children, Component, Context, Html, MouseEvent,
    Properties,
};

const WORDS: &str = include_str!("../vendor/kotus-sanalista_v1/kotus-sanalista_v1.xml");

fn parse_words(input: &str) -> Vec<Vec<char>> {
    let parts = input.split("<kotus-sanalista>\n").collect::<Vec<&str>>();
    let words = parts[1].split("</kotus-sanalista>").collect::<Vec<&str>>();

    let mut word_list = Vec::new();

    for line in words[0].lines() {
        let (word, _): (String, String) = serde_scan::scan!("<st><s>{}</s>{}" <- line).unwrap();

        if word.chars().count() == 5 {
            word_list.push(word.to_uppercase().chars().collect());
        }
    }

    word_list
}

enum Msg {
    KeyPress(char),
    Backspace,
    Guess,
}

struct Model {
    word_list: Vec<Vec<char>>,
    word: Vec<char>,
    is_guessing: bool,
    is_winner: bool,
    present_characters: HashSet<char>,
    correct_characters: HashSet<(char, usize)>,
    absent_characters: HashSet<char>,
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

impl Model {
    fn map_character_state(&self, character: char, index: usize) -> Option<&'static str> {
        if self.correct_characters.contains(&(character, index)) {
            Some("correct")
        } else if self.absent_characters.contains(&character) {
            Some("absent")
        } else if self.present_characters.contains(&character) {
            Some("present")
        } else {
            None
        }
    }

    fn map_keyboard_state(&self, character: char) -> Option<&'static str> {
        if self.correct_characters.iter().any(|(c, _)| *c == character) {
            Some("correct")
        } else if self.absent_characters.contains(&character) {
            Some("absent")
        } else if self.present_characters.contains(&character) {
            Some("present")
        } else {
            None
        }
    }
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
            present_characters: HashSet::new(),
            correct_characters: HashSet::new(),
            absent_characters: HashSet::new(),
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
            Msg::Guess => {
                if self.guesses[self.current_guess].len() != 5 {
                    return false;
                }

                if !self.word_list.contains(&self.guesses[self.current_guess]) {
                    return false;
                }

                self.is_winner = self.guesses[self.current_guess] == self.word;

                for (index, character) in self.guesses[self.current_guess].iter().enumerate() {
                    if self.word[index] == *character {
                        self.correct_characters.insert((*character, index));
                    }

                    if self.word.contains(character) {
                        self.present_characters.insert(*character);
                    } else {
                        self.absent_characters.insert(*character);
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
                    // <div class="subtitle">{ "Sana: "}{ self.word.iter().collect::<String>() }</div>
                </header>

                <div class="board-container">
                    <div class="board">
                        { self.guesses.iter().enumerate().map(|(g_index, guess)| html! {
                            <div class="row">
                                { (0..5).map(|c_index| html! {
                                    <div class={classes!(
                                        "tile",
                                        guess.get(c_index).and_then(|c| self.map_character_state(*c, c_index)),
                                        if g_index == self.current_guess { Some("current") } else { None }
                                    )}>
                                        { guess.get(c_index).unwrap_or(&' ') }
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
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                    </div>
                    <div>
                        {
                            keyboard[1].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                    </div>
                    <div>
                        {
                            keyboard[2].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                        <button class={classes!("keyboard-button")}
                            onclick={link.callback(move |_| Msg::Backspace)}>{ "<x]" }</button>
                        <button onclick={link.callback(|_| Msg::Guess)}>{ "ARVAA" }</button>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
