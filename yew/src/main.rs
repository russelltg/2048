use twenty_48::{Direction, GameState, Tile};
use yew::prelude::*;

struct Model {
    gs: GameState,
}

impl Component for Model {
    type Message = Direction;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            gs: GameState::new_from_entropy(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, dir: Self::Message) -> bool {
        self.gs.do_move(dir);
        self.gs.spawn_tile();

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();

        let r = self.gs.rows();
        let rows = r.iter().map(|&r| {
            html! {
                <tr>
                    {for r.iter().map(|t| html! {
                        <td><div class={t.map(|t| format!("value_{}", t.exponent())).unwrap_or("empty".into())}>{if let Some(t) = t { html!{t.as_u32()} } else { "".into() }}</div></td>
                    })}
                </tr>
            }
        });

        html! {
            <div tabindex="0" onkeydown={link.batch_callback(|e: KeyboardEvent| {
                log::info!("ev={:?}", e);
                match e.code().as_str() {
                    "ArrowLeft" => Some(Direction::Left),
                    "ArrowRight" => Some(Direction::Right),
                    "ArrowDown" => Some(Direction::Down),
                    "ArrowUp" => Some(Direction::Up),
                    _ => None,
                }
            })}>
                // <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                // <p>{ self.value }</p>
                <table class="game" >
                    { for rows }
                </table>
                // <TileComponent tile={self.gs.rows()[0][0]} />
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
