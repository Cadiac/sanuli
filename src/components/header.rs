use yew::prelude::*;

use crate::manager::GameMode;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_toggle_menu_cb: Callback<MouseEvent>,
    pub on_toggle_help_cb: Callback<MouseEvent>,
    pub game_mode: GameMode,
    pub streak: usize,
    pub daily_word_number: usize,
}

#[function_component(Header)]
pub fn header(props: &Props) -> Html {

    let on_toggle_help_cb = props.on_toggle_help_cb.clone();
    let onclick_help = {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            on_toggle_help_cb.emit(e);
        })
    };

    let on_toggle_menu_cb = props.on_toggle_menu_cb.clone();
    let onclick_menu = {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            on_toggle_menu_cb.emit(e);
        })
    };

    html! {
        <header>
            <nav onclick={onclick_help} class="title-icon">{"?"}</nav>
            {
                if let GameMode::DailyWord(_) = props.game_mode {
                    html! { <h1 class="title">{format!("Päivän sanuli #{}", props.daily_word_number)}</h1> }
                } else if props.streak > 0 {
                    html! { <h1 class="title">{format!("Sanuli — Putki: {}", props.streak)}</h1> }
                } else {
                    html! { <h1 class="title">{ "Sanuli" }</h1>}
                }
            }
            <nav onclick={onclick_menu} class="title-icon">{"≡"}</nav>
        </header>
    }
}
