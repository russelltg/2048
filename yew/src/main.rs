use twenty_48::{Direction, GameState};
use web_sys::{window, HtmlElement};
use yew::prelude::*;

enum Action {
    Move(Direction),
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
    TouchMove(TouchEvent),
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

    debug: String,
}

impl Model {
    fn save(&self) {
        window()
            .unwrap()
            .local_storage()
            .unwrap()
            .unwrap()
            .set_item("game", &serde_json::to_string(&self.gs).unwrap())
            .unwrap()
    }
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let gs = if let Ok(Some(name)) = window()
            .unwrap()
            .local_storage()
            .unwrap()
            .unwrap()
            .get_item("game")
        {
            serde_json::from_str(&name).ok()
        } else {
            None
        }
        .unwrap_or_else(|| GameState::new_from_entropy());

        Self {
            prev: gs.clone(),
            gs,
            container: NodeRef::default(),
            touch_start: None,
            debug: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, dir: Self::Message) -> bool {
        match dir {
            Action::Move(dir) => {
                if self.gs.can_move(dir) {
                    self.prev = self.gs.clone();
                    self.gs.do_move(dir);
                    self.gs.spawn_tile_with_dir(dir);
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

                true
            }
            Action::NewGame => {
                self.gs = GameState::new_from_entropy();
                self.prev = self.gs.clone();
                self.save();
                true
            }
            Action::TouchMove(te) => {
                let (x, y) = match self.touch_start {
                    Some((x, y)) => (x, y),
                    None => return false,
                };

                let t = te.touches().get(0).unwrap();
                let dx = t.client_x() - x;
                let dy = t.client_y() - y;

                if dx.abs() > 100 && dy.abs() < 50 {
                    ctx.link().send_message(Action::Move(if dx.is_negative() {
                        Direction::Left
                    } else {
                        Direction::Right
                    }));
                    self.touch_start = None;
                }
                if dy.abs() > 100 && dx.abs() < 50 {
                    ctx.link().send_message(Action::Move(if dy.is_negative() {
                        Direction::Up
                    } else {
                        Direction::Down
                    }));
                    self.touch_start = None;
                }

                false
            }
            Action::TouchEnd(_) => {
                log::info!("touch end");
                self.touch_start = None;
                true
            }
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

        let onkeydown = link.batch_callback(|e: KeyboardEvent| match e.code().as_str() {
            "ArrowLeft" => Some(Direction::Left.into()),
            "ArrowRight" => Some(Direction::Right.into()),
            "ArrowDown" => Some(Direction::Down.into()),
            "ArrowUp" => Some(Direction::Up.into()),
            "KeyU" => Some(Action::Undo),
            "KeyN" => Some(Action::NewGame),
            _ => None,
        });

        let ontouchstart = link.callback(|e: TouchEvent| Action::TouchStart(e));
        let ontouchend = link.callback(|e: TouchEvent| Action::TouchEnd(e));
        let ontouchmove = link.callback(|e: TouchEvent| Action::TouchMove(e));

        let lost = self.gs.lost();

        html! {
            <div ref={self.container.clone()} class="container" tabindex="0" onkeydown={onkeydown} ontouchstart={ontouchstart} ontouchend={ontouchend} ontouchmove={ontouchmove}>
                <div class="game">
                    <table>
                        { for rows }
                    </table>
                    { if lost { html! { <span class="lost_banner">{ "you lost" }</span> } } else { "".into() } }
                </div>
                <button onclick={link.callback(|_| Action::Undo)}>{ "Undo (u)" }</button>
                <button onclick={link.callback(|_| Action::NewGame)}>{ "New Game (n)" }</button>
                <span>{self.debug.clone()}</span>
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
    yew::Renderer::<Model>::new().render();
}
