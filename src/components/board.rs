use yew::prelude::*;

use crate::manager::TileState;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub is_guessing: bool,
    pub is_reset: bool,
    pub is_hidden: bool,

    pub guesses: Vec<Vec<(char, TileState)>>,
    pub previous_guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,
    pub max_guesses: usize,
    pub word_length: usize,
}

#[function_component(Board)]
pub fn board(props: &Props) -> Html {
    html! {
        <>
            {
                if !props.previous_guesses.is_empty() && props.is_reset {
                    html! {
                        <PreviousBoard
                            guesses={props.previous_guesses.clone()}
                            max_guesses={props.max_guesses}
                            word_length={props.word_length}
                        />
                    }
                } else {
                    html! {}
                }
            }
            <div class={classes!(
                props.is_reset.then(|| "slide-in"),
                props.is_reset.then(|| format!("slide-in-{}", props.previous_guesses.len())),
                format!("board-{}", props.max_guesses))}>{
                    props.guesses.iter().enumerate().map(|(row, guess)| {
                        let is_current_row = row == props.current_guess && props.is_guessing;

                        html! {
                            <div class={format!("row-{}", props.word_length)}>
                                {
                                    (0..props.word_length).map(|tile_index| {
                                        let (character, tile_state) = guess
                                            .get(tile_index)
                                            .unwrap_or(&(' ', TileState::Unknown));

                                        html! {
                                            <div class={classes!(
                                                "tile",
                                                tile_state.to_string(),
                                                is_current_row.then(|| Some("current"))
                                            )}>
                                                {
                                                    if props.is_hidden {
                                                        ' '
                                                    } else {
                                                        *character
                                                    }
                                                }
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct PreviousBoardProps {
    pub guesses: Vec<Vec<(char, TileState)>>,
    pub max_guesses: usize,
    pub word_length: usize,
}

#[function_component(PreviousBoard)]
pub fn previous_board(props: &PreviousBoardProps) -> Html {
    html! {
        <div class={classes!("slide-out", format!("slide-out-{}", props.guesses.len()), format!("board-{}", props.max_guesses))}>
            { props.guesses.iter().map(|guess| {
                html! {
                    <div class={format!("row-{}", props.word_length)}>
                        {
                            (0..props.word_length).map(|tile_index| {
                                let (character, tile_state) = guess
                                    .get(tile_index)
                                    .unwrap_or(&(' ', TileState::Unknown));

                                html! {
                                    <div class={classes!("tile", tile_state.to_string())}>
                                        { character }
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                }
            }).collect::<Html>() }
        </div>
    }
}
