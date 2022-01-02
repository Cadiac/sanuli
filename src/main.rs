use rand::seq::SliceRandom;
use wasm_bindgen::{prelude::Closure, JsCast};
use gloo::utils as gloo_utils;
use std::collections::HashMap;
use std::collections::HashSet;
use yew::{classes, html, Component, Context, Html, KeyboardEvent};

const WORDS: &str = include_str!("../word-list.txt");
const ALLOWED_KEYS: [char; 29] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];
const EMPTY: char = '\u{00a0}';
const FORMS_LINK_TEMPLATE: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";

fn parse_words(words: &str) -> Vec<Vec<char>> {
    words.lines().map(|word| word.chars().collect()).collect()
}

enum Msg {
    KeyPress(char),
    Backspace,
    Enter,
    Guess,
    NewGame,
    ToggleHelp,
    ToggleMenu,
}

struct Model {
    word_list: Vec<Vec<char>>,
    word: Vec<char>,

    is_guessing: bool,
    is_winner: bool,
    is_unknown: bool,
    message: String,

    is_help_visible: bool,
    is_menu_visible: bool,

    present_characters: HashSet<char>,
    correct_characters: HashSet<(char, usize)>,
    absent_characters: HashSet<char>,

    guesses: [Vec<char>; 6],
    current_guess: usize,
    streak: usize,

    keyboard_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

impl Model {
    fn character_state_mappings(&self, guess: &[char]) -> [Option<&'static str>; 5] {
        let mut mappings = [Some("absent"); 5];
        let mut correct_counts: HashMap<char, i32> = HashMap::new();
        let mut present_counts: HashMap<char, i32> = HashMap::new();

        for (index, character) in guess.iter().enumerate() {
            if self.correct_characters.contains(&(*character, index)) {
                *correct_counts.entry(*character).or_insert(0) += 1;
                mappings[index] = Some("correct");
            }
        }

        for (index, character) in guess.iter().enumerate() {
            if self.correct_characters.contains(&(*character, index)) {
                continue;
            }
            if self.present_characters.contains(character) {
                let correct_count = correct_counts.entry(*character).or_insert(0);
                let present_count = present_counts.entry(*character).or_insert(0);
                *present_count += 1;

                let character_present_in_word_count =
                    self.word.iter().filter(|c| *c == character).count() as i32;
                let is_found_all = *correct_count == character_present_in_word_count;
                if !is_found_all && *present_count - *correct_count <= character_present_in_word_count {
                    mappings[index] = Some("present");
                }
            }
        }

        mappings
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
            is_unknown: false,

            is_menu_visible: false,
            is_help_visible: false,

            message: EMPTY.to_string(),
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
            streak: 0,
            keyboard_listener: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let window: web_sys::Window = web_sys::window().expect("window not available");

        let cb = ctx.link().batch_callback(|e: KeyboardEvent| {
            if e.key().chars().count() == 1 {
                let key = e.key().to_uppercase().chars().next().unwrap();
                if ALLOWED_KEYS.contains(&key) && !e.ctrl_key() && !e.alt_key() && !e.meta_key() {
                    e.prevent_default();
                    Some(Msg::KeyPress(key))
                } else {
                    None
                }
            } else if e.key() == "Backspace" {
                e.prevent_default();
                Some(Msg::Backspace)
            } else if e.key() == "Enter" {
                e.prevent_default();
                Some(Msg::Enter)
            } else {
                None
            }
        });

        let listener =
            Closure::<dyn Fn(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| cb.emit(e)));

        window
            .add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
            .unwrap();
        self.keyboard_listener = Some(listener);
    }

    fn destroy(&mut self, _: &Context<Self>) {
        // remove the keyboard listener
        if let Some(listener) = self.keyboard_listener.take() {
            let window: web_sys::Window = web_sys::window().expect("window not available");
            window
                .remove_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
                .unwrap();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyPress(c) => {
                if !self.is_guessing || self.guesses[self.current_guess].len() >= 5 {
                    return false;
                }

                self.is_unknown = false;
                self.message = EMPTY.to_string();
                self.guesses[self.current_guess].push(c);

                true
            }
            Msg::Backspace => {
                if !self.is_guessing || self.guesses[self.current_guess].is_empty() {
                    return false;
                }

                self.is_unknown = false;
                self.message = EMPTY.to_string();
                self.guesses[self.current_guess].pop();

                true
            }

            Msg::Enter => {
                let link = ctx.link();

                if !self.is_guessing {
                    link.send_message(Msg::NewGame);
                } else {
                    link.send_message(Msg::Guess);
                }

                false
            }
            Msg::Guess => {
                if self.guesses[self.current_guess].len() != 5 {
                    self.message = String::from("Liian vähän kirjaimia!");
                    return true;
                }

                if !self.word_list.contains(&self.guesses[self.current_guess]) {
                    self.is_unknown = true;
                    return true;
                }

                self.is_unknown = false;
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

                if self.is_winner {
                    self.is_guessing = false;
                    self.streak += 1;
                    self.message = String::from("Löysit sanan! 🥳");
                } else if self.current_guess == 5 {
                    self.is_guessing = false;
                    self.message = format!("Sana oli \"{}\"", self.word.iter().collect::<String>());
                    self.streak = 0;
                } else {
                    self.message = EMPTY.to_string();
                    self.current_guess += 1;
                }

                true
            }
            Msg::NewGame => {
                self.word = self.word_list.choose(&mut rand::thread_rng()).unwrap().clone();

                self.is_guessing = true;
                self.is_winner = false;
                self.is_unknown = false;

                self.message = EMPTY.to_string();

                self.current_guess = 0;
                self.guesses = [
                    Vec::with_capacity(5),
                    Vec::with_capacity(5),
                    Vec::with_capacity(5),
                    Vec::with_capacity(5),
                    Vec::with_capacity(5),
                    Vec::with_capacity(5),
                ];

                self.correct_characters = HashSet::new();
                self.present_characters = HashSet::new();
                self.absent_characters = HashSet::new();

                true
            },
            Msg::ToggleHelp => {
                self.is_help_visible = !self.is_help_visible;

                true
            }
            Msg::ToggleMenu => {
                self.is_menu_visible = !self.is_menu_visible;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let keyboard = vec![
            vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å'],
            vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'],
            vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
        ];

        html! {
            <div class="game">
                <header>
                    <nav onclick={link.callback(|_| Msg::ToggleHelp)} class="title-icon">{"?"}</nav>
                    {
                        if self.streak > 0 {
                            html! { <h1 class="title">{format!("Sanuli — Putki: {}", self.streak)}</h1> }
                        } else {
                            html! { <h1 class="title">{ "Sanuli" }</h1>}
                        }
                    }
                    <nav onclick={link.callback(|_| Msg::ToggleMenu)} class="title-icon">{EMPTY}</nav>
                </header>
                <div class="board-container">
                    <div class="board">
                        { self.guesses.iter().enumerate().map(|(guess_index, guess)| {
                            let mappings = self.character_state_mappings(guess);

                            html! {
                                <div class="row">
                                    {
                                        (0..5).map(|char_index| html! {
                                        <div class={classes!(
                                            "tile",
                                            if self.is_guessing && guess_index == self.current_guess {
                                                guess.get(char_index).and_then(|c| self.map_keyboard_state(*c))
                                            } else {
                                                mappings[char_index]
                                            },
                                            if self.is_guessing && guess_index == self.current_guess { Some("current") } else { None }
                                        )}>
                                            { guess.get(char_index).unwrap_or(&' ') }
                                        </div>
                                    }).collect::<Html>() }
                                </div>
                            }
                        }).collect::<Html>() }
                    </div>
                </div>

                <div class="keyboard">
                    <div class="message">{
                        if self.is_unknown {
                            html! {
                                <span>{"Ei sanulistalla, "}
                                    <a href={format!("{}{}",
                                        FORMS_LINK_TEMPLATE,
                                        self.guesses[self.current_guess].iter().collect::<String>().to_lowercase())}
                                       target="_blank">{ "ehdota?" }
                                    </a>
                                </span>
                            }
                        } else {
                            html! { <span>{ &self.message }</span> }
                        }
                    }
                    </div>
                    <div class="keyboard-row">
                        {
                            keyboard[0].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                        <div class="spacer" />
                    </div>
                    <div class="keyboard-row">
                        <div class="spacer" />
                        {
                            keyboard[1].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="keyboard-row">
                        <div class="spacer" />
                        <div class="spacer" />
                        {
                            keyboard[2].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                        <button class={classes!("keyboard-button")}
                            onclick={link.callback(|_| Msg::Backspace)}>{ "⌫" }</button>
                        {
                            if self.is_guessing {
                                html! {
                                    <button class={classes!("keyboard-button")}
                                            onclick={link.callback(|_| Msg::Guess)}>
                                        { "ARVAA" }
                                    </button>
                                }
                            } else {
                                html! {
                                    <button class={classes!("keyboard-button", "correct")}
                                            onclick={link.callback(|_| Msg::NewGame)}>
                                        { "UUSI?" }
                                    </button>
                                }
                            }
                        }
                        <div class="spacer" />
                        <div class="spacer" />
                    </div>
                </div>

                {
                    if self.is_help_visible {
                        html! {
                            <div class="modal">
                                <span onclick={link.callback(|_| Msg::ToggleHelp)} class="modal-close">{"✖"}</span>
                                <p>{"Arvaa kätketty "}<i>{"sanuli"}</i>{" kuudella yrityksellä."}</p>
                                <p>{"Jokaisen yrityksen jälkeen arvatut kirjaimet vaihtavat väriään."}</p>

                                <div class="row example">
                                    <div class={classes!("tile", "correct")}>{"K"}</div>
                                    <div class={classes!("tile", "absent")}>{"O"}</div>
                                    <div class={classes!("tile", "present")}>{"I"}</div>
                                    <div class={classes!("tile", "absent")}>{"R"}</div>
                                    <div class={classes!("tile", "absent")}>{"A"}</div>
                                </div>

                                <p><span class="present">{"Keltainen"}</span>{": kirjain löytyy kätketystä sanasta, mutta on arvauksessa väärällä paikalla."}</p>
                                <p><span class="correct">{"Vihreä"}</span>{": kirjain on arvauksessa oikealla paikalla."}</p>
                                <p><span class="absent">{"Harmaa"}</span>{": kirjain ei löydy sanasta."}</p>

                                <p>
                                    {"Käytetyn sanalistan pohjalla on Kotimaisten kielten keskuksen (Kotus) julkaisemaa "}
                                    <a href="https://creativecommons.org/licenses/by/3.0/deed.fi" target="_blank">{"\"CC Nimeä 3.0 Muokkaamaton\""}</a>
                                    {" lisensoitu nykysuomen sanalista, josta on poimittu ne viisikirjaimiset sanat, jotka eivät sisällä vieraskielisiä kirjaimia. "}
                                    {"Sanalistaa on muokattu käyttäjien ehdotusten mukaan, ja voit jättää omat ehdotuksesi sanuihin "}
                                    <a href={FORMS_LINK_TEMPLATE}>{"täällä"}</a>
                                    {"."}
                                </p>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }
}

fn main() {
    // yew::start_app::<Model>();
    let app_element = gloo_utils::document()
        .get_element_by_id("app")
        .expect("a #app element");

    yew::start_app_in_element::<Model>(app_element);
}
