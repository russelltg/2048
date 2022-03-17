#![feature(const_option, int_log)]

use std::{
    fmt::{self, Display},
    io::stdout,
    num::NonZeroU32,
};

use crossterm::{
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Color, Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
    Command, ExecutableCommand,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Standard, Uniform};

#[derive(Clone)]
struct GameState {
    nums: [Option<Tile>; 16],
    rng: StdRng,
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
}

// first is coming from the direction that's being swpied in
// like 1 2 3 4 swiped left would give 1 2 3 4
fn movable<'a>(mut iter: impl Iterator<Item = &'a Option<Tile>>) -> bool {
    let mut prev = iter.next().unwrap();
    for cur in iter {
        if prev.is_none() || *prev == *cur {
            return true;
        }
        prev = cur;
    }
    return false;
}

impl GameState {
    fn new_from_seed(seed: u64) -> Self {
        Self::new(StdRng::seed_from_u64(seed))
    }

    fn new_from_entropy() -> Self {
        Self::new(StdRng::from_entropy())
    }

    fn new(rng: StdRng) -> Self {
        let mut s = GameState {
            nums: [None; 16],
            rng,
        };

        s.spawn_tile();
        s.spawn_tile();

        s
    }

    fn spawn_tile(&mut self) {
        let t = self.random_open_tile().unwrap();
        self.nums[t] = Some(self.rng.gen())
    }

    fn lost(&self) -> bool {
        Direction::ALL.iter().all(|d| !self.can_move(*d))
    }

    fn rows(&self) -> [[Option<Tile>; 4]; 4] {
        [
            self.nums[0..4].try_into().unwrap(),
            self.nums[4..8].try_into().unwrap(),
            self.nums[8..12].try_into().unwrap(),
            self.nums[12..16].try_into().unwrap(),
        ]
    }

    fn cols(&self) -> [[Option<Tile>; 4]; 4] {
        [
            [self.nums[0], self.nums[1], self.nums[2], self.nums[3]],
            [self.nums[4], self.nums[5], self.nums[6], self.nums[7]],
            [self.nums[8], self.nums[9], self.nums[10], self.nums[11]],
            [self.nums[12], self.nums[13], self.nums[14], self.nums[15]],
        ]
    }

    // fn rows_mut(&mut self) -> [[&mut Option<Tile>; 3]; 3] {
    //     let [a, b, c, d, e, f, g, h, i] = &mut self.nums;
    //     [[a, b, c], [d, e, f], [g, h, i]]
    // }

    // fn cols_mut(&mut self) -> [[&mut Option<Tile>; 3]; 3] {
    //     let [a, b, c, d, e, f, g, h, i] = &mut self.nums;
    //     [[a, d, g], [b, e, h], [c, f, i]]
    // }

    fn can_move(&self, direction: Direction) -> bool {
        // match direction {
        //     Direction::Up => self.cols().iter().any(|col| movable(col.iter())),
        //     Direction::Down => self.cols().iter().any(|col| movable(col.iter().rev())),
        //     Direction::Left => self.rows().iter().any(|col| movable(col.iter())),
        //     Direction::Right => self.rows().iter().any(|col| movable(col.iter().rev())),
        // }
        let (dperp, dpar, start): (i32, i32, i32) = match direction {
            Direction::Up => (4, 1, 0),
            Direction::Down => (-4, 1, 12),
            Direction::Left => (1, 4, 0),
            Direction::Right => (-1, 4, 3),
        };

        for par_idx in 0..4 {
            let s = start + par_idx * dpar;
            for perp_idx in 0..3 {
                let idx = s + perp_idx * dperp;

                for seekidx in 1..4 - perp_idx {
                    let n = (idx + seekidx * dperp) as usize;
                    if self.nums[n].is_some() {
                        if self.nums[idx as usize] == self.nums[n] {
                            return true;
                        } else if self.nums[idx as usize].is_none() {
                            return true;
                        } else {
                            break; // something in the way
                        }
                    }
                }
            }
        }
        return false;
    }

    fn do_move(&mut self, direction: Direction) {
        let (dperp, dpar, start): (i32, i32, i32) = match direction {
            Direction::Up => (4, 1, 0),
            Direction::Down => (-4, 1, 12),
            Direction::Left => (1, 4, 0),
            Direction::Right => (-1, 4, 3),
        };

        for par_idx in 0..4 {
            let s = start + par_idx * dpar;
            for perp_idx in 0..3 {
                let idx = s + perp_idx * dperp;

                for seekidx in 1..4 - perp_idx {
                    let n = (idx + seekidx * dperp) as usize;
                    if self.nums[n].is_some() {
                        if self.nums[idx as usize] == self.nums[n] {
                            self.nums[idx as usize] =
                                Some(self.nums[idx as usize].as_mut().unwrap().double());
                            self.nums[n] = None;
                            break;
                        } else if self.nums[idx as usize].is_none() {
                            self.nums[idx as usize] = self.nums[n];
                            self.nums[n] = None;
                        } else {
                            break; // something in the way
                        }
                    }
                }
            }
        }
    }

    fn random_open_tile(&mut self) -> Option<usize> {
        let open_tiles = self.nums.iter().filter(|t| t.is_none()).count();
        if open_tiles == 0 {
            None
        } else {
            Some(
                self.nums
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.is_none())
                    .nth(self.rng.sample(Uniform::new(0, open_tiles)))
                    .unwrap()
                    .0,
            )
        }
    }

    pub fn from_list(arg: [i32; 16]) -> Self {
        let mut nums = [None; 16];
        for (i, n) in arg.iter().enumerate() {
            if *n != -1 {
                nums[i] = Some(Tile(NonZeroU32::new(n.checked_log2().unwrap()).unwrap()));
            }
        }

        Self {
            nums,
            rng: StdRng::from_entropy(),
        }
    }
}

impl Command for GameState {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        for row in self.rows().iter() {
            for tile in row.iter() {
                match tile {
                    Some(tile) => write!(f, "|{: ^5}", tile.styled())?,
                    None => write!(f, "|{: ^5}", " ")?,
                }
            }
            write!(f, "|\r\n")?;
        }
        Ok(())
    }
}

// which power of two. NonZero because two is the lowest
#[derive(Copy, Clone, PartialEq, Eq)]
struct Tile(NonZeroU32);

impl Tile {
    const TWO: Tile = Tile(NonZeroU32::new(1).unwrap());
    const FOUR: Tile = Tile::TWO.double();

    const fn double(&self) -> Tile {
        Tile(NonZeroU32::new(self.0.get() + 1).unwrap())
    }

    fn as_u32(&self) -> u32 {
        2_u32.pow(self.0.get())
    }

    fn styled(&self) -> impl Display {
        format!("{: ^5}", self.as_u32()).with(match self.as_u32() {
            2 => Color::White,
            4 => Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }, // orange
            8 => Color::DarkYellow,
            16 => Color::Magenta,
            32 => Color::Green,
            64 => Color::Blue,
            _ => Color::White,
        })
    }
}

impl Distribution<Tile> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tile {
        if rng.sample(rand::distributions::Bernoulli::new(0.2).unwrap()) {
            Tile::TWO
        } else {
            Tile::FOUR
        }
    }
}

fn main() {
    let mut game = GameState::new_from_entropy();
    let mut prev_state = None;

    enable_raw_mode().unwrap();

    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend).unwrap();
    let mut stdout = stdout();

    'gameloop: loop {
        // stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(&game).unwrap();
        stdout.execute(Print("\n\n")).unwrap();

        if game.lost() {
            break 'gameloop;
        }

        let dir = match read().unwrap() {
            Event::Key(k) => match (k.code, k.modifiers) {
                (KeyCode::Left, KeyModifiers::NONE) => Direction::Left,
                (KeyCode::Right, KeyModifiers::NONE) => Direction::Right,
                (KeyCode::Up, KeyModifiers::NONE) => Direction::Up,
                (KeyCode::Down, KeyModifiers::NONE) => Direction::Down,
                (KeyCode::Char('u'), KeyModifiers::NONE) => {
                    if let Some(prev) = prev_state.take() {
                        game = prev;
                    }
                    continue 'gameloop;
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => break 'gameloop,
                _ => {
                    println!("{:?}", k);
                    continue 'gameloop;
                }
            },
            ev => {
                println!("{:?}", ev);
                continue 'gameloop;
            }
        };

        if game.can_move(dir) {
            prev_state = Some(game.clone());
            game.do_move(dir);
            game.spawn_tile();
        }
    }

    disable_raw_mode().unwrap();
}

#[cfg(test)]
mod test {
    use crate::{Direction, GameState};

    // | 128 | 64  | 32  |  8  |
    // |  8  |  4  |  8  |  4  |
    // |     |     |     |     |
    // |     |     |     |     |
    #[test]
    fn testcase1() {
        let gs = GameState::from_list([128, 64, 32, 8, 8, 4, 8, 4, -1, -1, -1, -1, -1, -1, -1, -1]);
        assert!(!gs.can_move(Direction::Right));
    }
}
