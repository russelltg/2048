use crate::{Direction, GameState};

pub fn solver_up_right_left_down(gs: &mut GameState) {
    while !gs.lost() {
        for d in [
            Direction::Up,
            Direction::Right,
            Direction::Left,
            Direction::Down,
        ] {
            if gs.can_move(d) {
                gs.do_move(d);
                gs.spawn_tile();
                break;
            }
        }
    }
}

pub fn solver_snake(gs: &mut GameState) {
    while !gs.lost() {
        // println!("{gs}");
        // stdin().read(&mut [0; 1024]).unwrap();
        let priority = if gs.can_move_row(0) {
            [
                Direction::Up,
                Direction::Left,
                Direction::Right,
                Direction::Down,
            ]
        } else if gs.can_move_row(1) {
            [
                Direction::Up,
                Direction::Right,
                Direction::Left,
                Direction::Down,
            ]
        } else {
            [
                Direction::Up,
                Direction::Left,
                Direction::Right,
                Direction::Down,
            ]
        };

        for d in priority {
            if gs.can_move(d) {
                gs.do_move(d);
                gs.spawn_tile();
                break;
            }
        }
    }
}
