use yew::prelude::*;

pub enum Msg {}

pub struct Timer {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub duration: u32,
    pub is_paused: bool,
}

impl Component for Timer {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_paused = if ctx.props().is_paused {
            "paused"
        } else {
            "running"
        };

        html! {
            <div class="bar-outline">
                <div class="bar" style={format!(
                    "animation: depletingBar {}s linear; animation-play-state: {}",
                    &ctx.props().duration,
                    is_paused)}
                />
            </div>
        }
    }
}
