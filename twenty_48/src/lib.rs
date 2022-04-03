#![feature(const_option, int_log)]

pub mod solvers;

use std::{
    fmt::{self, Display},
    io::{stdin, stdout, Read},
    num::NonZeroU32,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Standard, Uniform};

#[derive(Clone)]
pub struct GameState {
    nums: [Option<Tile>; 16],
    rng: StdRng,
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
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
    pub fn new_from_seed(seed: u64) -> Self {
        Self::new(StdRng::seed_from_u64(seed))
    }

    pub fn new_from_entropy() -> Self {
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

    pub fn spawn_tile(&mut self) {
        let t = self.random_open_tile().unwrap();
        self.nums[t] = Some(self.rng.gen())
    }

    pub fn lost(&self) -> bool {
        Direction::ALL.iter().all(|d| !self.can_move(*d))
    }

    pub fn rows(&self) -> [[Option<Tile>; 4]; 4] {
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

    pub fn can_move_col(&self, column: i32) -> bool {
        self.can_move_colrow(column, Direction::Up) || self.can_move_colrow(column, Direction::Down)
    }

    pub fn can_move_row(&self, row: i32) -> bool {
        self.can_move_colrow(row, Direction::Left) || self.can_move_colrow(row, Direction::Right)
    }

    pub fn can_move_colrow(&self, colrow: i32, direction: Direction) -> bool {
        let (dperp, dpar, start): (i32, i32, i32) = match direction {
            Direction::Up => (4, 1, 0),
            Direction::Down => (-4, 1, 12),
            Direction::Left => (1, 4, 0),
            Direction::Right => (-1, 4, 3),
        };

        let s = start + colrow * dpar;
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
        false
    }

    pub fn can_move(&self, direction: Direction) -> bool {
        (0..4).any(|colrow| self.can_move_colrow(colrow, direction))

        // let (dperp, dpar, start): (i32, i32, i32) = match direction {
        //     Direction::Up => (4, 1, 0),
        //     Direction::Down => (-4, 1, 12),
        //     Direction::Left => (1, 4, 0),
        //     Direction::Right => (-1, 4, 3),
        // };

        // for par_idx in 0..4 {
        //     let s = start + par_idx * dpar;
        //     for perp_idx in 0..3 {
        //         let idx = s + perp_idx * dperp;

        //         for seekidx in 1..4 - perp_idx {
        //             let n = (idx + seekidx * dperp) as usize;
        //             if self.nums[n].is_some() {
        //                 if self.nums[idx as usize] == self.nums[n] {
        //                     return true;
        //                 } else if self.nums[idx as usize].is_none() {
        //                     return true;
        //                 } else {
        //                     break; // something in the way
        //                 }
        //             }
        //         }
        //     }
        // }
        // false
    }

    pub fn do_move(&mut self, direction: Direction) {
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

    pub fn max(&self) -> u32 {
        self.nums
            .iter()
            .filter_map(|t| t.map(|t| t.as_u32()))
            .max()
            .unwrap()
    }

    pub fn print(&self) {
        println!("{self}");
    }
    fn print_row(f: &mut impl fmt::Write, row: &[Option<Tile>]) -> fmt::Result {
        for tile in row.iter() {
            match tile {
                Some(tile) => write!(f, "|{: ^5}", tile.as_u32())?,
                None => write!(f, "|{: ^5}", " ")?,
            }
        }
        Ok(())
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.rows().iter() {
            GameState::print_row(f, row)?;
            write!(f, "|\n")?;
        }
        Ok(())
    }
}

// which power of two. NonZero because two is the lowest
#[derive(Copy, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "yew", derive(yew::ImplicitClone))]
pub struct Tile(NonZeroU32);

impl Tile {
    const TWO: Tile = Tile(NonZeroU32::new(1).unwrap());
    const FOUR: Tile = Tile::TWO.double();

    const fn double(&self) -> Tile {
        Tile(NonZeroU32::new(self.0.get() + 1).unwrap())
    }

    pub fn as_u32(&self) -> u32 {
        2_u32.pow(self.0.get())
    }

    pub fn exponent(&self) -> u32 {
        self.0.get()
    }
}

impl Distribution<Tile> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tile {
        if rng.sample(rand::distributions::Bernoulli::new(0.9).unwrap()) {
            Tile::TWO
        } else {
            Tile::FOUR
        }
    }
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
