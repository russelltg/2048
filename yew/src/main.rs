use twenty_48::{Direction, GameState};
use web_sys::{HtmlElement, window};
use yew::prelude::*;

enum Action {
    Move(Direction),
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
    NewGame,
    Undo,
}

impl From<Direction> for Action {
    fn from(d: Direction) -> Self {
        Action::Move(d)
    }
}

struct Model {
    prev: GameState,
    gs: GameState,
    container: NodeRef,
    touch_start: Option<(i32, i32)>,
}

impl Model {
    fn save(&self) {
        window().unwrap().local_storage().unwrap().unwrap().set_item("game", &serde_json::to_string(&self.gs).unwrap()).unwrap()
    }
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let gs = if let Ok(Some(name)) = window().unwrap().local_storage().unwrap().unwrap().get_item("game") {
            serde_json::from_str(&name).ok()
        } else {
            None
        }.unwrap_or_else(|| GameState::new_from_entropy());

        Self {
            prev: gs.clone(),
            gs,
            container: NodeRef::default(),
            touch_start: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, dir: Self::Message) -> bool {
        match dir {
            Action::Move(dir) => {
                if self.gs.can_move(dir) {
                    self.prev = self.gs.clone();
                    self.gs.do_move(dir);
                    self.gs.spawn_tile();
                    self.save();
                    true
                } else {
                    false
                }
            }
            Action::Undo => {
                self.gs = self.prev.clone();
                self.save();
                true
            }
            Action::TouchStart(ts) => {
                let tl = ts.touches();
                if tl.length() != 1 {
                    return false;
                }

                let t = tl.get(0).unwrap();

                self.touch_start = Some((t.client_x(), t.client_y()));

                false
            }
            Action::NewGame => {
                self.gs = GameState::new_from_entropy();
                self.prev = self.gs.clone();
                self.save();
                true
            }
            Action::TouchEnd(_) => todo!(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();

        let r = self.gs.rows();
        let rows = r.iter().map(|&r| {
            html! {
                <tr>
                    {for r.iter().map(|t| html! {
                        <td>
                            <div class={t.map(|t| format!("value_{}", t.exponent())).unwrap_or("empty".into())}>
                                {if let Some(t) = t { html!{t.as_u32()} } else { "".into() }}
                            </div>
                        </td>
                    })}
                </tr>
            }
        });

        let onkeydown = link.batch_callback(|e: KeyboardEvent| {
            log::info!("ev={:?}", e);
            match e.code().as_str() {
                "ArrowLeft" => Some(Direction::Left.into()),
                "ArrowRight" => Some(Direction::Right.into()),
                "ArrowDown" => Some(Direction::Down.into()),
                "ArrowUp" => Some(Direction::Up.into()),
                "KeyU" => Some(Action::Undo),
                "KeyN" => Some(Action::NewGame),
                _ => None,
            }
        });

        let ontouchstart = link.callback(|e: TouchEvent| Action::TouchStart(e));

        let lost = self.gs.lost();

        html! {
            <div ref={self.container.clone()} class="container" tabindex="0" onkeydown={onkeydown} ontouchstart={ontouchstart}>
                <span class="game">
                    <table>
                        { for rows }
                    </table>
                    { if lost { html! { <span class="lost_banner">{ "you lost" }</span> } } else { "".into() } }
                </span>
                <button onclick={link.callback(|_| Action::Undo)}>{ "Undo (u)" }</button>
                <button onclick={link.callback(|_| Action::NewGame)}>{ "New Game (n)" }</button>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        self.container
            .cast::<HtmlElement>()
            .unwrap()
            .focus()
            .unwrap();
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
