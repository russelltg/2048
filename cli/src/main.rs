use std::{
    fmt::{self},
    io::{stdin, stdout, Read},
};

use crossterm::{
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Color, Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
    Command, ExecutableCommand,
};
use twenty_48::{solvers, Direction, GameState, Tile};

struct GsCommand<'a>(&'a GameState);

impl<'a> Command for GsCommand<'a> {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        for row in self.0.rows().iter() {
            print_row(f, row)?;
            write!(f, "|\r\n")?;
        }
        Ok(())
    }
}

fn styled(t: &Tile) -> impl fmt::Display {
    format!("{: ^5}", t.as_u32()).with(match t.as_u32() {
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

fn print_row(f: &mut impl fmt::Write, row: &[Option<Tile>]) -> fmt::Result {
    for tile in row.iter() {
        match tile {
            Some(tile) => write!(f, "|{: ^5}", styled(tile))?,
            None => write!(f, "|{: ^5}", " ")?,
        }
    }
    Ok(())
}

fn play_interactive() {
    let mut game = GameState::new_from_entropy();
    let mut prev_state = None;

    enable_raw_mode().unwrap();

    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend).unwrap();
    let mut stdout = stdout();

    'gameloop: loop {
        // stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(&GsCommand(&game)).unwrap();
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
            game.spawn_tile_with_dir(dir);
        }
    }

    disable_raw_mode().unwrap();
}

fn solve(solver: fn(&mut GameState)) {
    let mut scores = Vec::new();
    loop {
        let mut game = GameState::new_from_entropy();

        solver(&mut game);
        scores.push(game.max());

        game.print();
        println!(
            "{}",
            2.0_f64.powf(scores.iter().map(|i| i.ilog(2)).sum::<u32>() as f64 / scores.len() as f64)
        );
        stdin().read(&mut [0; 1024]).unwrap();
    }
}

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    match arg.as_str() {
        "i" | "interactive" => play_interactive(),
        "urld" => solve(solvers::solver_up_right_left_down),
        "snake" => solve(solvers::solver_snake),
        c => panic!("unrecognized command {c}"),
    }
}
