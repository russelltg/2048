use serde::de::DeserializeOwned;
use twenty_48::{Direction, GameState};
use web_sys::{js_sys::Date, window, HtmlDialogElement, HtmlElement};
use yew::prelude::*;

enum Action {
    Move(Direction),
    TouchStart(TouchEvent),
    TouchEnd,
    TouchMove(TouchEvent),
    NewGame,
    Undo,
    OpenScoreboard,
    CloseScoreboard,
}

impl From<Direction> for Action {
    fn from(d: Direction) -> Self {
        Action::Move(d)
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Scoreboard {
    top: [Option<(u64, String)>; 5], // (score, date)
}

struct Model {
    scoreboard: Scoreboard,
    prev: GameState,
    gs: GameState,
    container: NodeRef,
    scoreboard_dialog: NodeRef,
    touch_start: Option<(i32, i32)>,

    debug: String,
}

impl Model {
    const LS_KEY_GAME: &str = "game";
    const LS_KEY_SCOREBOARD: &str = "scoreboard";

    fn save(&self) {
        let storage = &window().unwrap().local_storage().unwrap().unwrap();
        storage
            .set_item(
                Model::LS_KEY_GAME,
                &serde_json::to_string(&self.gs).unwrap(),
            )
            .unwrap();
        storage
            .set_item(
                Model::LS_KEY_SCOREBOARD,
                &serde_json::to_string(&self.scoreboard).unwrap(),
            )
            .unwrap();
    }
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let gs = load_from_storage(Model::LS_KEY_GAME).unwrap_or_else(GameState::new_from_entropy);
        let scoreboard = load_from_storage(Model::LS_KEY_SCOREBOARD).unwrap_or_default();

        Self {
            prev: gs.clone(),
            gs,
            scoreboard,
            container: NodeRef::default(),
            scoreboard_dialog: NodeRef::default(),
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
                self.scoreboard.add(self.gs.score());
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
            Action::TouchEnd => {
                log::info!("touch end");
                self.touch_start = None;
                true
            }
            Action::OpenScoreboard => {
                self.scoreboard_dialog
                    .cast::<HtmlDialogElement>()
                    .unwrap()
                    .show_modal()
                    .unwrap();
                true
            }
            Action::CloseScoreboard => {
                self.scoreboard_dialog
                    .cast::<HtmlDialogElement>()
                    .unwrap()
                    .close();
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

        let scoreboard_rows = self.scoreboard.top.iter().flatten().map(|(score, date)| {
            html! {
                <tr><td>{score}</td><td>{date}</td></tr>
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
        let ontouchend = link.callback(|_e: TouchEvent| Action::TouchEnd);
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
                <div class="score">
                    { "Score: " } { self.gs.score() }
                </div>
                <button onclick={link.callback(|_| Action::Undo)}>{ "Undo (u)" }</button>
                <button onclick={link.callback(|_| Action::NewGame)}>{ "New Game (n)" }</button>
                <button onclick={link.callback(|_| Action::OpenScoreboard)}>{ "Scoreboard..." }</button>
                <dialog ref={self.scoreboard_dialog.clone()} class="scoreboard">
                    <table>
                        { for scoreboard_rows }
                    </table>
                    <button autofocus=true onclick={link.callback(|_| Action::CloseScoreboard)}>{ "Close" }</button>
                </dialog>
                <span>{self.debug.clone()}</span>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.container
            .cast::<HtmlElement>()
            .unwrap()
            .focus()
            .unwrap();
    }
}

fn load_from_storage<T: DeserializeOwned>(key: &str) -> Option<T> {
    if let Ok(Some(t)) = window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .get_item(key)
    {
        serde_json::from_str(&t).ok()
    } else {
        None
    }
}

impl Scoreboard {
    fn add(&mut self, new_score: u64) {
        for i in 0..self.top.len() {
            if let Some((score, _)) = self.top[i] {
                if new_score <= score {
                    continue;
                }
            }

            // this is a new high score, shift down and insert
            self.top[i..].rotate_right(1);
            self.top[i] = Some((
                new_score,
                Date::new_0().to_date_string().as_string().unwrap(),
            ));

            return;
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Model>::new().render();
}
