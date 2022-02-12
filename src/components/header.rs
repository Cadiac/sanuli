use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_toggle_menu_cb: Callback<MouseEvent>,
    pub on_toggle_help_cb: Callback<MouseEvent>,
    pub title: String,
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
                <h1 class="title">{&props.title}</h1>
            <nav onclick={onclick_menu} class="title-icon">{"≡"}</nav>
        </header>
    }
}
