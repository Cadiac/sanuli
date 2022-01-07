use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Window};
use yew::{classes, html, Component, Context, Html, KeyboardEvent};

mod state;
use state::{State, GameMode};

const ALLOWED_KEYS: [char; 29] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];

const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const FORMS_LINK_TEMPLATE_DEL: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Poistoa&entry.560255602=";
const DICTIONARY_LINK_TEMPLATE: &str = "https://www.kielitoimistonsanakirja.fi/#/";

const KEYBOARD_0: [char; 11] = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å'];
const KEYBOARD_1: [char; 11] = ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'];
const KEYBOARD_2: [char; 7] = ['Z', 'X', 'C', 'V', 'B', 'N', 'M'];

pub enum Msg {
    KeyPress(char),
    Backspace,
    Enter,
    Guess,
    NewGame,
    ToggleHelp,
    ToggleMenu,
    ChangeGameMode(GameMode),
    ChangePreviousGameMode,
    ChangeWordLength(usize),
}

pub struct App {
    state: State,
    is_help_visible: bool,
    is_menu_visible: bool,
    keyboard_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut initial_state = Self {
            state: State::new(state::DEFAULT_WORD_LENGTH, state::DEFAULT_MAX_GUESSES),
            is_help_visible: false,
            is_menu_visible: false,
            keyboard_listener: None
        };

        if initial_state.state.rehydrate().is_err() {
            // Reinitialize and just continue with defaults
            initial_state.state = State::new(state::DEFAULT_WORD_LENGTH, state::DEFAULT_MAX_GUESSES);
        }

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
            Msg::KeyPress(c) => self.state.push_character(c),
            Msg::Backspace => self.state.pop_character(),
            Msg::Enter => {
                let link = ctx.link();

                if !self.state.is_guessing {
                    if self.state.game_mode == GameMode::DailyWord {
                        link.send_message(Msg::ChangePreviousGameMode);
                    } else {
                        link.send_message(Msg::NewGame);
                    }
                } else {
                    link.send_message(Msg::Guess);
                }

                true
            }
            Msg::Guess => self.state.submit_guess(),
            Msg::NewGame => self.state.create_new_game(),
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
                self.state.change_word_length(new_length);
                self.state.create_new_game()
            }
            Msg::ChangeGameMode(new_mode) => {
                self.state.change_game_mode(new_mode);
                self.state.create_new_game()
            }
            Msg::ChangePreviousGameMode => {
                self.state.change_game_mode(self.state.previous_game_mode.clone());
                self.state.create_new_game()
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div class="game">
                <header>
                    <nav onmousedown={link.callback(|_| Msg::ToggleHelp)} class="title-icon">{"?"}</nav>
                    {
                        if self.state.game_mode == GameMode::DailyWord {
                            html! { <h1 class="title">{format!("Päivän sanuli #{}", self.state.get_daily_word_index() + 1)}</h1> }
                        } else if self.state.streak > 0 {
                            html! { <h1 class="title">{format!("Sanuli — Putki: {}", self.state.streak)}</h1> }
                        } else {
                            html! { <h1 class="title">{ "Sanuli" }</h1>}
                        }
                    }
                    <nav onmousedown={link.callback(|_| Msg::ToggleMenu)} class="title-icon">{"≡"}</nav>
                </header>

                <div class="board-container">
                    {
                        if !self.state.previous_guesses.is_empty() && self.state.is_reset {
                            html! {
                                <div class={classes!("slide-out", format!("slide-out-{}", self.state.previous_guesses.len()), format!("board-{}", self.state.max_guesses))}>
                                    { self.state.previous_guesses.iter().enumerate().map(|(guess_index, guess)| {
                                        let mappings = self.state.map_guess_row(guess, guess_index);
                                        html! {
                                            <div class={format!("row-{}", self.state.word_length)}>
                                                {(0..self.state.word_length).map(|char_index| html! {
                                                    <div class={classes!("tile", mappings[char_index])}>
                                                        { guess.get(char_index).unwrap_or(&' ') }
                                                    </div>
                                                }).collect::<Html>() }
                                            </div>
                                        }
                                    }).collect::<Html>() }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    <div class={classes!(
                        self.state.is_reset.then(|| "slide-in"),
                        self.state.is_reset.then(|| format!("slide-in-{}", self.state.previous_guesses.len())),
                        format!("board-{}", self.state.max_guesses))}>{
                            self.state.guesses.iter().enumerate().map(|(guess_index, guess)| {
                                let mappings = self.state.map_guess_row(guess, guess_index);
                                if guess_index == self.state.current_guess {
                                    html! {
                                        <div class={format!("row-{}", self.state.word_length)}>
                                            {
                                                (0..self.state.word_length).map(|char_index| html! {
                                                    <div class={classes!(
                                                        "tile",
                                                        if self.state.is_guessing {
                                                            guess.get(char_index).and_then(|c| self.state.map_current_row(&(*c, char_index)))
                                                        } else {
                                                            mappings[char_index]
                                                        },
                                                        self.state.is_guessing.then(|| Some("current"))
                                                    )}>
                                                        { guess.get(char_index).unwrap_or(&' ') }
                                                    </div>
                                                }).collect::<Html>()
                                            }
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class={format!("row-{}", self.state.word_length)}>
                                            {(0..self.state.word_length).map(|char_index| html! {
                                                <div class={classes!("tile", mappings[char_index])}>
                                                    { guess.get(char_index).unwrap_or(&' ') }
                                                </div>
                                            }).collect::<Html>() }
                                        </div>
                                    }
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                <div class="keyboard">
                    <div class="message">
                        { &self.state.message }
                        <div class="message-small">{{
                            if self.state.is_unknown {
                                let last_guess = self.state.guesses[self.state.current_guess].iter().collect::<String>().to_lowercase();
                                html! {
                                    <a href={format!("{}{}", FORMS_LINK_TEMPLATE_ADD, last_guess)}
                                        target="_blank">{ "Ehdota lisäystä?" }
                                    </a>
                                }
                            } else if !self.state.is_winner & !self.state.is_guessing {
                                let word = self.state.word.iter().collect::<String>().to_lowercase();
                                html! {
                                    <>
                                        <a href={format!("{}{}?searchMode=all", DICTIONARY_LINK_TEMPLATE, word)}
                                            target="_blank">{ "Sanakirja" }
                                        </a>
                                        {" | "}
                                        <a href={format!("{}{}", FORMS_LINK_TEMPLATE_DEL, word)}
                                            target="_blank">{ "Ehdota poistoa?" }
                                        </a>
                                    </>
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
                                    <button data-nosnippet="" class={classes!("keyboard-button", self.state.map_keyboard_state(key))}
                                            onmousedown={link.callback(move |_| Msg::KeyPress(*key))}>
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
                                    <button data-nosnippet="" class={classes!("keyboard-button", self.state.map_keyboard_state(key))}
                                        onmousedown={link.callback(move |_| Msg::KeyPress(*key))}>
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
                                    <button data-nosnippet="" class={classes!("keyboard-button", self.state.map_keyboard_state(key))}
                                        onmousedown={link.callback(move |_| Msg::KeyPress(*key))}>{ key }</button>
                                }).collect::<Html>()
                        }
                        <button data-nosnippet="" class={classes!("keyboard-button")}
                            onmousedown={link.callback(|_| Msg::Backspace)}>{ "⌫" }</button>
                        {
                            if self.state.is_guessing {
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button")}
                                        onmousedown={link.callback(|_| Msg::Guess)}>
                                        { "ARVAA" }
                                    </button>
                                }
                            } else if self.state.game_mode == GameMode::DailyWord {
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button", "correct")}
                                        onmousedown={link.callback(|_| Msg::ChangePreviousGameMode)}>
                                        { "TAKAISIN" }
                                    </button>
                                }
                            } else {
                                html! {
                                    <button data-nosnippet="" class={classes!("keyboard-button", "correct")}
                                        onmousedown={link.callback(|_| Msg::NewGame)}>
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
                                <span onmousedown={link.callback(|_| Msg::ToggleHelp)} class="modal-close">{"✖"}</span>
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
                                    {" lisensoitu nykysuomen sanalista, josta on poimittu ne sanat, jotka sisältävät vain kirjaimia A-Ö. "}
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
                                <span onmousedown={link.callback(|_| Msg::ToggleMenu)} class="modal-close">{"✖"}</span>
                                <div>
                                    <p class="title">{"Sanulien pituus:"}</p>
                                    <div class="select-container">
                                        <button class={classes!("select", (self.state.word_length == 5).then(|| Some("select-active")))}
                                            onmousedown={link.callback(|_| Msg::ChangeWordLength(5))}>
                                            {"5 merkkiä"}
                                        </button>
                                        <button class={classes!("select", (self.state.word_length == 6).then(|| Some("select-active")))}
                                            onmousedown={link.callback(|_| Msg::ChangeWordLength(6))}>
                                            {"6 merkkiä"}
                                        </button>
                                    </div>
                                </div>
                                <div>
                                    <p class="title">{"Pelimuoto:"}</p>
                                    <div class="select-container">
                                        <button class={classes!("select", (self.state.game_mode == GameMode::Classic).then(|| Some("select-active")))}
                                            onmousedown={link.callback(|_| Msg::ChangeGameMode(GameMode::Classic))}>
                                            {"Peruspeli"}
                                        </button>
                                        <button class={classes!("select", (self.state.game_mode == GameMode::Relay).then(|| Some("select-active")))}
                                            onmousedown={link.callback(|_| Msg::ChangeGameMode(GameMode::Relay))}>
                                            {"Sanuliketju"}
                                        </button>
                                        // <button class={classes!("select", (self.game_mode == GameMode::DailyWord).then(|| Some("select-active")))}
                                        //     onclick={link.callback(|_| Msg::ChangeGameMode(GameMode::DailyWord))}>
                                        //     {"Päivän sanuli"}
                                        // </button>
                                    </div>
                                </div>
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
    yew::start_app::<App>();
}
