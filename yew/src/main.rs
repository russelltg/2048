use num_format::{Locale, ToFormattedString};
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

#[derive(Default)]
struct Scoreboard([Option<(u64, String)>; 5]);

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct StatsHistory(Vec<PastGameDatapoint>);

#[derive(Default)]
struct Stats {
    history: StatsHistory,

    lifetime_points: u64,

    scoreboard: Scoreboard,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PastGameDatapoint {
    date: String,
    score: u64,
}

struct Model {
    stats: Stats,
    prev: GameState,
    gs: GameState,
    container: NodeRef,
    scoreboard_dialog: NodeRef,
    touch_start: Option<(i32, i32)>,

    debug: String,
}

impl Model {
    const LS_KEY_GAME: &str = "game";
    const LS_KEY_HISTORY: &str = "history";

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
                Model::LS_KEY_HISTORY,
                &serde_json::to_string(&self.stats.history).unwrap(),
            )
            .unwrap();
    }

    fn scoreboard(&self) -> Html {
        let scoreboard_rows = self
            .stats
            .scoreboard
            .0
            .iter()
            .flatten()
            .map(|(score, date)| {
                html! {
                    <tr><td>{score.to_formatted_string(&Locale::en)}</td><td>{date}</td></tr>
                }
            });

        html! {
            <div>
                <h2>{"Scoreboard"}</h2>
                <table>
                    { for scoreboard_rows }
                </table>
            </div>
        }
    }

    fn histogram(&self) -> Html {
        let min = self
            .stats
            .history
            .0
            .iter()
            .map(|t| t.score)
            .min()
            .unwrap_or_default();
        let max = self
            .stats
            .history
            .0
            .iter()
            .map(|t| t.score)
            .max()
            .unwrap_or_default();

        const BINS: u64 = 10;
        const BIN_MULT: u64 = 100;
        const MAX_TABLE_ROWS: u64 = 20;

        let min_round_down = min / BIN_MULT * BIN_MULT;
        let max_round_up = max.next_multiple_of(BIN_MULT);

        let bin_width =
            (((max_round_up - min_round_down) / BIN_MULT).div_ceil(BINS) * BIN_MULT).max(100); // make sure it's at least 100

        let mut bins = [0_u64; BINS as usize];
        for d in &self.stats.history.0 {
            bins[((d.score - min_round_down) / bin_width) as usize] += 1;
        }
        let max_bin = *bins.iter().max().unwrap();
        let each_row_means = max_bin.div_ceil(MAX_TABLE_ROWS).max(1);

        let rows = max_bin.div_ceil(each_row_means);

        html! {
            <div>
                <h2>{"Histogram"}</h2>
                <table class="histogram">
                    <tr>
                        <td />
                        { for bins.iter().map(|b| html! { <td><div class="hist-header">{b}</div></td>  }) }
                    </tr>
                    { for (1..=rows).rev().map(|i| {
                        let threshold = i * each_row_means;
                        html!{
                            <tr>
                                <td>{if i % 3 == 0 { threshold.to_string() } else { String::new() }}</td>
                                { for bins.iter().map(|&b| html!{
                                    { if b >= threshold { html!{<td class="hist-fill" />}} else {html!{<td />}} }
                                })}
                            </tr>
                        }
                    })}
                    <tr>
                        <td />
                        { for (0..bins.len() as u64).map(|i| html!{
                            <td><div class="hist-footer">
                                {format!("{}-{}", min_round_down + i * bin_width, min_round_down + (i + 1) * bin_width - 1)}
                            </div></td>
                        })}
                    </tr>
                </table>
            </div>
        }
    }

    fn scoreboard_elem(&self) -> Option<HtmlDialogElement> {
        self.scoreboard_dialog.cast::<HtmlDialogElement>()
    }
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let gs = load_from_storage(Model::LS_KEY_GAME).unwrap_or_else(GameState::new_from_entropy);
        let stats = Stats::new(load_from_storage(Model::LS_KEY_HISTORY).unwrap_or_default());

        Self {
            prev: gs.clone(),
            gs,
            stats,
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
                let score = self.gs.score();
                if score > 10 {
                    self.stats
                        .on_game_finish(score, Date::new_0().to_date_string().as_string().unwrap());
                }
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
                self.scoreboard_elem().unwrap().show_modal().unwrap();
                true
            }
            Action::CloseScoreboard => {
                self.scoreboard_elem().unwrap().close();
                false
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
        let ontouchend = link.callback(|_e: TouchEvent| Action::TouchEnd);
        let ontouchmove = link.callback(|e: TouchEvent| Action::TouchMove(e));

        let lost = self.gs.lost();
        let score = self.gs.score();

        let stats_contents = if self.scoreboard_elem().map(|d| d.open()).unwrap_or(false) {
            let scoreboard = self.scoreboard();
            let hist = self.histogram();
            html! {
                <div>
                    <div>
                        { "Lifetime points: " } { (self.stats.lifetime_points + score).to_formatted_string(&Locale::en) }
                    </div>
                    { scoreboard }
                    { hist }
                    <button autofocus=true onclick={link.callback(|_| Action::CloseScoreboard)}>{ "Close" }</button>
                </div>
            }
        } else {
            "".into_html()
        };

        html! {
            <div ref={self.container.clone()} class="container" tabindex="0" onkeydown={onkeydown} ontouchstart={ontouchstart} ontouchend={ontouchend} ontouchmove={ontouchmove}>
                <div class="game">
                    <table>
                        { for rows }
                    </table>
                    { if lost { html! { <span class="lost_banner">{ "you lost" }</span> } } else { "".into() } }
                </div>
                <div class="score">
                    { "Score: " } { score.to_formatted_string(&Locale::en) }
                </div>
                <button onclick={link.callback(|_| Action::Undo)}>{ "Undo (u)" }</button>
                <button onclick={link.callback(|_| Action::NewGame)}>{ "New Game (n)" }</button>
                <button onclick={link.callback(|_| Action::OpenScoreboard)}>{ "Stats..." }</button>
                <dialog ref={self.scoreboard_dialog.clone()} class="scoreboard">
                    { stats_contents }
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

impl Stats {
    fn new(history: StatsHistory) -> Stats {
        let lifetime_points: u64 = history.0.iter().map(|h| h.score).sum();
        let mut scoreboard = Scoreboard::default();
        for g in &history.0 {
            scoreboard.add(g.score, g.date.clone());
        }
        Self {
            history,
            scoreboard,
            lifetime_points,
        }
    }

    fn on_game_finish(&mut self, score: u64, date: String) {
        self.scoreboard.add(score, date.clone());
        self.history.0.push(PastGameDatapoint { score, date });
        self.lifetime_points += score;
    }
}

impl Scoreboard {
    fn add(&mut self, new_score: u64, date: String) {
        for i in 0..self.0.len() {
            if let Some((score, _)) = self.0[i] {
                if new_score <= score {
                    continue;
                }
            }

            // this is a new high score, shift down and insert
            self.0[i..].rotate_right(1);
            self.0[i] = Some((new_score, date));

            return;
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Model>::new().render();
}
