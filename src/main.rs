use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{window, Element, HtmlElement, Window};
use yew::{classes, html, Component, Context, Html, KeyboardEvent, NodeRef};

const WORDS: &str = include_str!("../word-list.txt");
const ALLOWED_KEYS: [char; 29] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];
const EMPTY: char = '\u{00a0}';
const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const FORMS_LINK_TEMPLATE_DEL: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Poistoa&entry.560255602=";
const DEFAULT_WORD_LENGTH: usize = 5;
const DEFAULT_MAX_GUESSES: usize = 6;

const KEYBOARD_0: [char; 11] = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å'];
const KEYBOARD_1: [char; 11] = ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'];
const KEYBOARD_2: [char; 7] = ['Z', 'X', 'C', 'V', 'B', 'N', 'M'];

const SUCCESS_EMOJIS: [&str; 8] = ["🥳", "🤩", "🤗", "🎉", "😊", "😺", "😎", "👏"];

fn parse_words(words: &str, word_length: usize) -> Vec<Vec<char>> {
    words
        .lines()
        .filter(|word| word.chars().count() == word_length)
        .map(|word| word.chars().collect())
        .collect()
}

enum Msg {
    KeyPress(char),
    Backspace,
    Enter,
    Guess,
    NewGame,
    NewGameTransition,
    ToggleHelp,
    ToggleMenu,
    ChangeWordLength(usize),
}

struct Model {
    word_list: Vec<Vec<char>>,
    word: Vec<char>,

    word_length: usize,
    max_guesses: usize,

    is_guessing: bool,
    is_winner: bool,
    is_unknown: bool,
    message: String,

    is_help_visible: bool,
    is_menu_visible: bool,

    present_characters: HashSet<char>,
    correct_characters: HashSet<(char, usize)>,
    absent_characters: HashSet<char>,

    guesses: Vec<Vec<char>>,
    previous_guesses: Vec<Vec<char>>,
    current_guess: usize,
    streak: usize,

    guesses_ref: NodeRef,
    keyboard_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

impl Model {
    fn character_state_mappings(&self, guess: &[char]) -> Vec<Option<&'static str>> {
        let mut mappings = vec![Some("absent"); self.word_length];
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
                if !is_found_all
                    && *present_count - *correct_count <= character_present_in_word_count
                {
                    mappings[index] = Some("present");
                }
            }
        }

        mappings
    }

    fn map_keyboard_state(&self, character: &char) -> Option<&'static str> {
        if self.correct_characters.iter().any(|(c, _)| c == character) {
            Some("correct")
        } else if self.absent_characters.contains(character) {
            Some("absent")
        } else if self.present_characters.contains(character) {
            Some("present")
        } else {
            None
        }
    }

    fn handle_guess(&mut self) -> bool {
        if self.guesses[self.current_guess].len() != self.word_length {
            self.message = "Liian vähän kirjaimia!".to_owned();
            return true;
        }

        if !self.word_list.contains(&self.guesses[self.current_guess]) {
            self.is_unknown = true;
            self.message = "Ei sanulistalla.".to_owned();
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
            self.message = format!(
                "Löysit sanan! {}",
                SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap()
            );
        } else if self.current_guess == self.max_guesses - 1 {
            self.is_guessing = false;
            self.message = format!("Sana oli \"{}\"", self.word.iter().collect::<String>());
            self.streak = 0;
        } else {
            self.message = EMPTY.to_string();
            self.current_guess += 1;
        }

        let _result = self.persist_guess();

        true
    }

    fn persist_guess(&mut self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        let local_storage = window.local_storage().expect("local storage not available");
        if let Some(local_storage) = local_storage {
            local_storage.set_item("streak", format!("{}", self.streak).as_str())?;
            local_storage.set_item("is_guessing", format!("{}", self.is_guessing).as_str())?;
            local_storage.set_item("is_winner", format!("{}", self.is_winner).as_str())?;
            local_storage.set_item("message", &self.message)?;
            local_storage.set_item("current_guess", format!("{}", self.current_guess).as_str())?;
            local_storage.set_item(
                "guesses",
                &self
                    .guesses
                    .iter()
                    .map(|guess| guess.iter().collect::<String>())
                    .collect::<Vec<String>>()
                    .join(","),
            )?;
        }

        Ok(())
    }

    fn persist_new_game(&mut self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        let local_storage = window.local_storage().expect("local storage not available");
        if let Some(local_storage) = local_storage {
            local_storage.set_item("word", &self.word.iter().collect::<String>())?;
            local_storage.set_item("word_length", format!("{}", self.word_length).as_str())?;
            local_storage.remove_item("is_guessing")?;
            local_storage.remove_item("is_winner")?;
            local_storage.remove_item("message")?;
            local_storage.remove_item("current_guess")?;
            local_storage.remove_item("guesses")?;
        }

        Ok(())
    }

    fn rehydrate(&mut self) -> Result<(), JsValue> {
        let window: Window = window().expect("window not available");
        if let Some(local_storage) = window.local_storage().expect("local storage not available") {
            let word_length_item = local_storage.get_item("word_length")?;
            if let Some(word_length_str) = word_length_item {
                if let Ok(word_length) = word_length_str.parse::<usize>() {
                    if word_length != self.word_length {
                        self.word_list = parse_words(WORDS, word_length);
                    }
                    self.word_length = word_length;
                }
            }

            let word = local_storage.get_item("word")?;
            if let Some(word) = word {
                self.word = word.chars().collect();
            } else {
                local_storage.set_item("word", &self.word.iter().collect::<String>())?;
            }

            let streak_item = local_storage.get_item("streak")?;
            if let Some(streak_str) = streak_item {
                if let Ok(streak) = streak_str.parse::<usize>() {
                    self.streak = streak;
                }
            }

            let is_guessing_item = local_storage.get_item("is_guessing")?;
            if let Some(is_guessing_str) = is_guessing_item {
                if let Ok(is_guessing) = is_guessing_str.parse::<bool>() {
                    self.is_guessing = is_guessing;
                }
            }

            let is_winner_item = local_storage.get_item("is_winner")?;
            if let Some(is_winner_str) = is_winner_item {
                if let Ok(is_winner) = is_winner_str.parse::<bool>() {
                    self.is_winner = is_winner;
                }
            }

            let message_item = local_storage.get_item("message")?;
            if let Some(message_str) = message_item {
                self.message = message_str;
            }

            let current_guess_item = local_storage.get_item("current_guess")?;
            if let Some(current_guess_str) = current_guess_item {
                if let Ok(current_guess) = current_guess_str.parse::<usize>() {
                    self.current_guess = current_guess;
                }
            }

            let guesses_item = local_storage.get_item("guesses")?;
            if let Some(guesses_str) = guesses_item {
                let previous_guesses = guesses_str.split(',').map(|guess| guess.chars().collect());

                for (guess_index, guess) in previous_guesses.enumerate() {
                    self.guesses[guess_index] = guess;

                    for (index, character) in self.guesses[guess_index].iter().enumerate() {
                        if self.word[index] == *character {
                            self.correct_characters.insert((*character, index));
                        }
                        if self.word.contains(character) {
                            self.present_characters.insert(*character);
                        } else {
                            self.absent_characters.insert(*character);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let word_list = parse_words(WORDS, DEFAULT_WORD_LENGTH);

        let word = word_list.choose(&mut rand::thread_rng()).unwrap().clone();
        let guesses = std::iter::repeat(Vec::with_capacity(DEFAULT_WORD_LENGTH))
            .take(DEFAULT_MAX_GUESSES)
            .collect::<Vec<_>>();

        let mut initial_state = Self {
            word,
            word_list,

            word_length: DEFAULT_WORD_LENGTH,
            max_guesses: DEFAULT_MAX_GUESSES,

            is_guessing: true,
            is_winner: false,
            is_unknown: false,

            is_menu_visible: false,
            is_help_visible: false,

            message: EMPTY.to_string(),
            present_characters: HashSet::new(),
            correct_characters: HashSet::new(),
            absent_characters: HashSet::new(),
            guesses,
            previous_guesses: Vec::new(),
            current_guess: 0,
            streak: 0,
            guesses_ref: NodeRef::default(),
            keyboard_listener: None,
        };

        let _result = initial_state.rehydrate();

        initial_state
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let window: Window = window().expect("window not available");

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
            let window: Window = window().expect("window not available");
            window
                .remove_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
                .unwrap();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyPress(c) => {
                if !self.is_guessing || self.guesses[self.current_guess].len() >= self.word_length {
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
            Msg::Guess => self.handle_guess(),
            Msg::NewGameTransition => {
                // https://aerotwist.com/blog/flip-your-animations/
                let el = self.guesses_ref.cast::<HtmlElement>().unwrap();

                let first = el.get_bounding_client_rect();

                let class_list = js_sys::Array::new();
                class_list.set(0, "move-to-first-row".into());

                el.class_list().add(&class_list).unwrap();

                let last = el.get_bounding_client_rect();

                let invert = first.top() - last.top();

                el.style()
                    .set_property("transform", &format!("translateY({}px)", invert))
                    .unwrap();

                let cb = Closure::wrap(Box::new(move || {
                    let class_list = js_sys::Array::new();
                    class_list.set(0, "animate-transition".into());

                    el.class_list().add(&class_list).unwrap();

                    el.style().set_property("transform", "").unwrap();
                }) as Box<dyn FnMut()>);

                window()
                    .expect("window not available")
                    .request_animation_frame(cb.as_ref().unchecked_ref())
                    .expect("should register `requestAnimationFrame`");

                // TODO: Maybe don't just leak this, keep in state?
                cb.forget();

                false
            }
            Msg::NewGame => {
                let previous_word = mem::replace(
                    &mut self.word,
                    self.word_list
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone(),
                );

                self.is_guessing = true;
                self.is_winner = false;
                self.is_unknown = false;

                self.message = EMPTY.to_string();

                self.previous_guesses = mem::replace(
                    &mut self.guesses,
                    std::iter::repeat(Vec::with_capacity(self.word_length))
                        .take(self.max_guesses)
                        .collect::<Vec<_>>(),
                );

                self.correct_characters = HashSet::new();
                self.present_characters = HashSet::new();
                self.absent_characters = HashSet::new();
                if previous_word.len() == self.word_length {
                    self.guesses[0] = previous_word;
                    self.current_guess = 1;

                    for (index, character) in self.guesses[0].iter().enumerate() {
                        if self.word[index] == *character {
                            self.correct_characters.insert((*character, index));
                        }
                        if self.word.contains(character) {
                            self.present_characters.insert(*character);
                        } else {
                            self.absent_characters.insert(*character);
                        }
                    }
                } else {
                    self.current_guess = 0;
                }

                let _result = self.persist_new_game();

                ctx.link().send_message(Msg::NewGameTransition);

                true
            }
            Msg::ToggleHelp => {
                self.is_help_visible = !self.is_help_visible;
                self.is_menu_visible = false;
                true
            }
            Msg::ToggleMenu => {
                self.is_menu_visible = !self.is_menu_visible;
                self.is_help_visible = false;
                true
            }
            Msg::ChangeWordLength(new_length) => {
                self.word_length = new_length;
                self.word_list = parse_words(WORDS, self.word_length);
                self.streak = 0;
                self.is_menu_visible = false;

                ctx.link().send_message(Msg::NewGame);

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

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
                    <nav onclick={link.callback(|_| Msg::ToggleMenu)} class="title-icon">{"≡"}</nav>
                </header>
                {
                    html! {
                        <div class={classes!("board-container", self.previous_guesses.is_empty().then(|| "hidden"))}>
                            <div ref={self.guesses_ref.clone()} class={format!("board-{}", self.max_guesses)}>
                                { self.guesses.iter().enumerate().map(|(guess_index, guess)| {
                                    let mappings = self.character_state_mappings(guess);

                                    if guess_index == self.current_guess {
                                        html! {
                                            <div class={format!("row-{}", self.word_length)}>
                                                {
                                                    (0..self.word_length).map(|char_index| html! {
                                                    <div class={classes!(
                                                        "tile",
                                                        if self.is_guessing {
                                                            guess.get(char_index).and_then(|c| self.map_keyboard_state(c))
                                                        } else {
                                                            mappings[char_index]
                                                        },
                                                        self.is_guessing.then(|| Some("current"))
                                                    )}>
                                                        { guess.get(char_index).unwrap_or(&' ') }
                                                    </div>
                                                }).collect::<Html>() }
                                            </div>
                                        }
                                    } else {
                                        html! {
                                            <div class={format!("row-{}", self.word_length)}>
                                                {(0..self.word_length).map(|char_index| html! {
                                                    <div class={classes!("tile", mappings[char_index])}>
                                                        { guess.get(char_index).unwrap_or(&' ') }
                                                    </div>
                                                }).collect::<Html>() }
                                            </div>
                                        }
                                    }
                                }).collect::<Html>() }
                            </div>
                        </div>
                    }
                }

                <div class="board-container">
                    <div class={format!("board-{}", self.max_guesses)}>
                        { self.guesses.iter().enumerate().map(|(guess_index, guess)| {
                            let mappings = self.character_state_mappings(guess);

                            if guess_index == self.current_guess {
                                html! {
                                    <div class={format!("row-{}", self.word_length)}>
                                        {
                                            (0..self.word_length).map(|char_index| html! {
                                            <div class={classes!(
                                                "tile",
                                                if self.is_guessing {
                                                    guess.get(char_index).and_then(|c| self.map_keyboard_state(c))
                                                } else {
                                                    mappings[char_index]
                                                },
                                                self.is_guessing.then(|| Some("current"))
                                            )}>
                                                { guess.get(char_index).unwrap_or(&' ') }
                                            </div>
                                        }).collect::<Html>() }
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class={format!("row-{}", self.word_length)}>
                                        {(0..self.word_length).map(|char_index| html! {
                                            <div class={classes!("tile", mappings[char_index])}>
                                                { guess.get(char_index).unwrap_or(&' ') }
                                            </div>
                                        }).collect::<Html>() }
                                    </div>
                                }
                            }
                        }).collect::<Html>() }
                    </div>
                </div>

                <div class="keyboard">
                    <div class="message">
                        { &self.message }
                        <div class="message-small">{{
                            let word = self.guesses[self.current_guess].iter().collect::<String>().to_lowercase();
                            if self.is_unknown {
                                html! {
                                    <a href={format!("{}{}", FORMS_LINK_TEMPLATE_ADD, word)}
                                        target="_blank">{ "Ehdota lisäystä?" }
                                    </a>
                                }
                            } else if !self.is_winner & !self.is_guessing {
                                html! {
                                    <a href={format!("{}{}", FORMS_LINK_TEMPLATE_DEL, word)}
                                        target="_blank">{ "Ehdota poistoa?" }
                                    </a>
                                }
                            } else {
                                html! {}
                            }
                        }}
                        </div>
                    </div>

                    <div class="keyboard-row">
                        {
                            KEYBOARD_0.iter().map(|key|
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                            onclick={link.callback(move |_| Msg::KeyPress(*key))}>
                                        { key }
                                    </button>
                                }).collect::<Html>()
                        }
                        <div class="spacer" />
                    </div>
                    <div class="keyboard-row">
                        <div class="spacer" />
                        {
                            KEYBOARD_1.iter().map(|key|
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                            onclick={link.callback(move |_| Msg::KeyPress(*key))}>
                                            { key }
                                    </button>
                                }).collect::<Html>()
                        }
                    </div>
                    <div class="keyboard-row">
                        <div class="spacer" />
                        <div class="spacer" />
                        {
                            KEYBOARD_2.iter().map(|key|
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                        onclick={link.callback(move |_| Msg::KeyPress(*key))}>{ key }</button>
                                }).collect::<Html>()
                        }
                        <button data-nosnippet="" class={classes!("keyboard-button")}
                            onclick={link.callback(|_| Msg::Backspace)}>{ "⌫" }</button>
                        {
                            if self.is_guessing {
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button")}
                                            onclick={link.callback(|_| Msg::Guess)}>
                                        { "ARVAA" }
                                    </button>
                                }
                            } else {
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button", "correct")}
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

                                <div class="row-5 example">
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
                                    {"Käytetyn sanalistan pohjana on Kotimaisten kielten keskuksen (Kotus) julkaisema "}
                                    <a href="https://creativecommons.org/licenses/by/3.0/deed.fi" target="_blank">{"\"CC Nimeä 3.0 Muokkaamaton\""}</a>
                                    {" lisensoitu nykysuomen sanalista, josta on poimittu ne viisikirjaimiset sanat, jotka sisältävät vain kirjaimia A-Ö. "}
                                    {"Sanalistaa muokataan jatkuvasti käyttäjien ehdotusten perusteella, ja voit jättää omat ehdotuksesi sanuihin "}
                                    <a href={FORMS_LINK_TEMPLATE_ADD}>{"täällä"}</a>
                                    {"."}
                                </p>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                {
                    if self.is_menu_visible {
                        html! {
                            <div class="modal">
                                <span onclick={link.callback(|_| Msg::ToggleMenu)} class="modal-close">{"✖"}</span>
                                <p class="title">{"Sanojen pituus:"}</p>
                                <button class={classes!("select", (self.word_length == 5).then(|| Some("select-active")))}
                                    onclick={link.callback(|_| Msg::ChangeWordLength(5))}>
                                    {"5 merkkiä"}
                                </button>
                                <button class={classes!("select", (self.word_length == 6).then(|| Some("select-active")))}
                                    onclick={link.callback(|_| Msg::ChangeWordLength(6))}>
                                    {"6 merkkiä"}
                                </button>
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
    yew::start_app::<Model>();
}
