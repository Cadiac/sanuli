use std::collections::HashMap;
use yew::prelude::*;

use crate::state::{TileState, GameMode};
use crate::Msg;

use crate::components::{message::Message};

const KEYBOARD_0: [char; 10] = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'];
const KEYBOARD_1: [char; 11] = ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'];
const KEYBOARD_2: [char; 7] = ['Z', 'X', 'C', 'V', 'B', 'N', 'M'];

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<Msg>,

    pub is_unknown: bool,
    pub is_winner: bool,
    pub is_guessing: bool,
    pub game_mode: GameMode,

    pub message: String,
    pub word: String,
    pub last_guess: String,

    pub keyboard: HashMap<char, TileState>,
}

#[function_component(Keyboard)]
pub fn keyboard(props: &Props) -> Html {
    let callback = props.callback.clone();
    let onbackspace = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        callback.emit(Msg::Backspace);
    });

    html! {
        <div class="keyboard">
            <Message
                message={props.message.clone()}
                is_unknown={props.is_unknown}
                is_winner={props.is_winner}
                is_guessing={props.is_guessing}
                last_guess={props.last_guess.clone()}
                word={props.word.clone()}
            />

            <div class="keyboard-row">
                {
                    KEYBOARD_0.iter().map(|key| {
                        let callback = props.callback.clone();
                        let onkeypress = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::KeyPress(*key));
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", props.keyboard.get(key).unwrap().to_string())}
                                onmousedown={onkeypress}>
                                { key }
                            </button>
                        }
                    }).collect::<Html>()
                }
                <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-backspace")} onmousedown={onbackspace}>
                    { "⌫" }
                </button>
            </div>
            <div class="keyboard-row">
                <div class="spacer" />
                {
                    KEYBOARD_1.iter().map(|key| {
                        let callback = props.callback.clone();
                        let onkeypress = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::KeyPress(*key));
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", props.keyboard.get(key).unwrap().to_string())}
                                onmousedown={onkeypress}>
                                { key }
                            </button>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div class="keyboard-row">
                <div class="spacer" />
                <div class="spacer" />
                <div class="spacer" />
                {
                    KEYBOARD_2.iter().map(|key| {
                        let callback = props.callback.clone();
                        let onkeypress = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::KeyPress(*key));
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", props.keyboard.get(key).unwrap().to_string())}
                                onmousedown={onkeypress}>{ key }</button>
                        }
                    }).collect::<Html>()
                }
                {
                    if props.is_guessing {
                        let callback = props.callback.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::Guess);
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-submit")}
                                onmousedown={onmousedown}>
                                { "ARVAA" }
                            </button>
                        }
                    } else if let GameMode::DailyWord(_) = props.game_mode {
                        let callback = props.callback.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::ChangePreviousGameMode);
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-submit", "correct")}
                                onmousedown={onmousedown}>
                                { "TAKAISIN" }
                            </button>
                        }
                    } else {
                        let callback = props.callback.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::NextWord);
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-submit", "correct")}
                                onmousedown={onmousedown}>
                                { "UUSI?" }
                            </button>
                        }
                    }
                }
                <div class="spacer" />
                <div class="spacer" />
            </div>
        </div>
    }
}
