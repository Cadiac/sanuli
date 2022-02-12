use yew::prelude::*;

use crate::manager::TileState;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub is_guessing: bool,
    pub is_reset: bool,
    pub is_hidden: bool,

    pub guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,
    pub max_guesses: usize,
    pub word_length: usize,
}

#[function_component(Board)]
pub fn quad_board(props: &Props) -> Html {
    html! {
        <div class="quad-container">
            <Board
                is_guessing={self.manager.game.is_guessing}
                is_reset={self.manager.game.is_reset}
                is_hidden={self.manager.game.is_hidden}
                guesses={self.manager.game.guesses.clone()}
                previous_guesses={Vec::new()}
                current_guess={self.manager.game.current_guess}
                max_guesses={self.manager.game.max_guesses}
                word_length={self.manager.game.word_length}
            />
            <Board
                is_guessing={self.manager.game.is_guessing}
                is_reset={self.manager.game.is_reset}
                is_hidden={self.manager.game.is_hidden}
                guesses={self.manager.game.guesses.clone()}
                previous_guesses={self.manager.game.previous_guesses.clone()}
                current_guess={self.manager.game.current_guess}
                max_guesses={self.manager.game.max_guesses}
                word_length={self.manager.game.word_length}
            />
            <Board
                is_guessing={self.manager.game.is_guessing}
                is_reset={self.manager.game.is_reset}
                is_hidden={self.manager.game.is_hidden}
                guesses={self.manager.game.guesses.clone()}
                previous_guesses={self.manager.game.previous_guesses.clone()}
                current_guess={self.manager.game.current_guess}
                max_guesses={self.manager.game.max_guesses}
                word_length={self.manager.game.word_length}
            />
            <Board
                is_guessing={self.manager.game.is_guessing}
                is_reset={self.manager.game.is_reset}
                is_hidden={self.manager.game.is_hidden}
                guesses={self.manager.game.guesses.clone()}
                previous_guesses={self.manager.game.previous_guesses.clone()}
                current_guess={self.manager.game.current_guess}
                max_guesses={self.manager.game.max_guesses}
                word_length={self.manager.game.word_length}
            />
        </div>
    }
}
