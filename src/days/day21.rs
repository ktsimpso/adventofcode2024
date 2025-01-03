use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_lines, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse, StringParse};
use ahash::AHashMap;
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    IterParser, Parser,
};
use clap::Args;
use itertools::Itertools;
use std::{iter::once, sync::LazyLock};

pub static DAY_21: LazyLock<CliProblem<Day21, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day21",
            "Finds the number of key presses to unlock a door",
            "Newline delimited list of desired door codes",
        )
        .with_part(
            "Computes the sum of the complexities with 2 intermediary robots",
            CommandLineArguments { n: 2 },
            vec![("sample.txt", 126384)],
        )
        .with_part(
            "Computes the sum of the complexities with 25 intermediary robots",
            CommandLineArguments { n: 25 },
            vec![],
        )
        .freeze()
    });

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(
        short,
        long,
        help = "The number of robots between you and the final robot"
    )]
    n: usize,
}

pub struct Day21(Vec<Vec<Code>>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day21, extra::Err<Rich<'a, char>>> {
    parse_lines(Code::parse().repeated().at_least(1).collect::<Vec<_>>())
        .map(Day21)
        .end()
}

#[derive(Debug, Clone, PartialEq, Eq, StringParse)]
enum Code {
    #[literal("0")]
    Zero,
    #[literal("1")]
    One,
    #[literal("2")]
    Two,
    #[literal("3")]
    Three,
    #[literal("4")]
    Four,
    #[literal("5")]
    Five,
    #[literal("6")]
    Six,
    #[literal("7")]
    Seven,
    #[literal("8")]
    Eight,
    #[literal("9")]
    Nine,
    #[literal("A")]
    Activate,
}

impl Code {
    fn get_shortest_sequence(&self, destination: &Self) -> Vec<Keypad> {
        match (self, destination) {
            (x, y) if x == y => vec![Keypad::Activate],
            (Code::Zero, Code::One) => vec![Keypad::Up, Keypad::Left, Keypad::Activate],
            (Code::Zero, Code::Two) => vec![Keypad::Up, Keypad::Activate],
            (Code::Zero, Code::Three) => vec![Keypad::Up, Keypad::Right, Keypad::Activate],
            (Code::Zero, Code::Four) => {
                vec![Keypad::Up, Keypad::Up, Keypad::Left, Keypad::Activate]
            }
            (Code::Zero, Code::Five) => vec![Keypad::Up, Keypad::Up, Keypad::Activate],
            (Code::Zero, Code::Six) => {
                vec![Keypad::Up, Keypad::Up, Keypad::Right, Keypad::Activate]
            }
            (Code::Zero, Code::Seven) => vec![
                Keypad::Up,
                Keypad::Up,
                Keypad::Up,
                Keypad::Left,
                Keypad::Activate,
            ],
            (Code::Zero, Code::Eight) => vec![Keypad::Up, Keypad::Up, Keypad::Up, Keypad::Activate],
            (Code::Zero, Code::Nine) => vec![
                Keypad::Up,
                Keypad::Up,
                Keypad::Up,
                Keypad::Right,
                Keypad::Activate,
            ],
            (Code::Zero, Code::Activate) => vec![Keypad::Right, Keypad::Activate],
            (Code::One, Code::Zero) => vec![Keypad::Right, Keypad::Down, Keypad::Activate],
            (Code::One, Code::Two) => vec![Keypad::Right, Keypad::Activate],
            (Code::One, Code::Three) => vec![Keypad::Right, Keypad::Right, Keypad::Activate],
            (Code::One, Code::Four) => vec![Keypad::Up, Keypad::Activate],
            (Code::One, Code::Five) => vec![Keypad::Up, Keypad::Right, Keypad::Activate],
            (Code::One, Code::Six) => {
                vec![Keypad::Up, Keypad::Right, Keypad::Right, Keypad::Activate]
            }
            (Code::One, Code::Seven) => vec![Keypad::Up, Keypad::Up, Keypad::Activate],
            (Code::One, Code::Eight) => {
                vec![Keypad::Up, Keypad::Up, Keypad::Right, Keypad::Activate]
            }
            (Code::One, Code::Nine) => vec![
                Keypad::Up,
                Keypad::Up,
                Keypad::Right,
                Keypad::Right,
                Keypad::Activate,
            ],
            (Code::One, Code::Activate) => {
                vec![Keypad::Right, Keypad::Right, Keypad::Down, Keypad::Activate]
            }
            (Code::Two, Code::Zero) => vec![Keypad::Down, Keypad::Activate],
            (Code::Two, Code::One) => vec![Keypad::Left, Keypad::Activate],
            (Code::Two, Code::Three) => vec![Keypad::Right, Keypad::Activate],
            (Code::Two, Code::Four) => vec![Keypad::Left, Keypad::Up, Keypad::Activate],
            (Code::Two, Code::Five) => vec![Keypad::Up, Keypad::Activate],
            (Code::Two, Code::Six) => vec![Keypad::Up, Keypad::Right, Keypad::Activate],
            (Code::Two, Code::Seven) => {
                vec![Keypad::Left, Keypad::Up, Keypad::Up, Keypad::Activate]
            }
            (Code::Two, Code::Eight) => vec![Keypad::Up, Keypad::Up, Keypad::Activate],
            (Code::Two, Code::Nine) => {
                vec![Keypad::Up, Keypad::Up, Keypad::Right, Keypad::Activate]
            }
            (Code::Two, Code::Activate) => vec![Keypad::Down, Keypad::Right, Keypad::Activate],
            (Code::Three, Code::Zero) => vec![Keypad::Down, Keypad::Left, Keypad::Activate],
            (Code::Three, Code::One) => vec![Keypad::Left, Keypad::Left, Keypad::Activate],
            (Code::Three, Code::Two) => vec![Keypad::Left, Keypad::Activate],
            (Code::Three, Code::Four) => {
                vec![Keypad::Left, Keypad::Left, Keypad::Up, Keypad::Activate]
            }
            (Code::Three, Code::Five) => vec![Keypad::Left, Keypad::Up, Keypad::Activate],
            (Code::Three, Code::Six) => vec![Keypad::Up, Keypad::Activate],
            (Code::Three, Code::Seven) => vec![
                Keypad::Left,
                Keypad::Left,
                Keypad::Up,
                Keypad::Up,
                Keypad::Activate,
            ],
            (Code::Three, Code::Eight) => {
                vec![Keypad::Left, Keypad::Up, Keypad::Up, Keypad::Activate]
            }
            (Code::Three, Code::Nine) => vec![Keypad::Up, Keypad::Up, Keypad::Activate],
            (Code::Three, Code::Activate) => vec![Keypad::Down, Keypad::Activate],
            (Code::Four, Code::Zero) => {
                vec![Keypad::Right, Keypad::Down, Keypad::Down, Keypad::Activate]
            }
            (Code::Four, Code::One) => vec![Keypad::Down, Keypad::Activate],
            (Code::Four, Code::Two) => vec![Keypad::Down, Keypad::Right, Keypad::Activate],
            (Code::Four, Code::Three) => {
                vec![Keypad::Down, Keypad::Right, Keypad::Right, Keypad::Activate]
            }
            (Code::Four, Code::Five) => vec![Keypad::Right, Keypad::Activate],
            (Code::Four, Code::Six) => vec![Keypad::Right, Keypad::Right, Keypad::Activate],
            (Code::Four, Code::Seven) => vec![Keypad::Up, Keypad::Activate],
            (Code::Four, Code::Eight) => vec![Keypad::Up, Keypad::Right, Keypad::Activate],
            (Code::Four, Code::Nine) => {
                vec![Keypad::Up, Keypad::Right, Keypad::Right, Keypad::Activate]
            }
            (Code::Four, Code::Activate) => vec![
                Keypad::Right,
                Keypad::Right,
                Keypad::Down,
                Keypad::Down,
                Keypad::Activate,
            ],
            (Code::Five, Code::Zero) => vec![Keypad::Down, Keypad::Down, Keypad::Activate],
            (Code::Five, Code::One) => vec![Keypad::Down, Keypad::Left, Keypad::Activate],
            (Code::Five, Code::Two) => vec![Keypad::Down, Keypad::Activate],
            (Code::Five, Code::Three) => vec![Keypad::Down, Keypad::Right, Keypad::Activate],
            (Code::Five, Code::Four) => vec![Keypad::Left, Keypad::Activate],
            (Code::Five, Code::Six) => vec![Keypad::Right, Keypad::Activate],
            (Code::Five, Code::Seven) => vec![Keypad::Left, Keypad::Up, Keypad::Activate],
            (Code::Five, Code::Eight) => vec![Keypad::Up, Keypad::Activate],
            (Code::Five, Code::Nine) => vec![Keypad::Up, Keypad::Right, Keypad::Activate],
            (Code::Five, Code::Activate) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Right, Keypad::Activate]
            }
            (Code::Six, Code::Zero) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Left, Keypad::Activate]
            }
            (Code::Six, Code::One) => {
                vec![Keypad::Down, Keypad::Left, Keypad::Left, Keypad::Activate]
            }
            (Code::Six, Code::Two) => vec![Keypad::Down, Keypad::Left, Keypad::Activate],
            (Code::Six, Code::Three) => vec![Keypad::Down, Keypad::Activate],
            (Code::Six, Code::Four) => vec![Keypad::Left, Keypad::Left, Keypad::Activate],
            (Code::Six, Code::Five) => vec![Keypad::Left, Keypad::Activate],
            (Code::Six, Code::Seven) => {
                vec![Keypad::Left, Keypad::Left, Keypad::Up, Keypad::Activate]
            }
            (Code::Six, Code::Eight) => vec![Keypad::Left, Keypad::Up, Keypad::Activate],
            (Code::Six, Code::Nine) => vec![Keypad::Up, Keypad::Activate],
            (Code::Six, Code::Activate) => vec![Keypad::Down, Keypad::Down, Keypad::Activate],
            (Code::Seven, Code::Zero) => vec![
                Keypad::Right,
                Keypad::Down,
                Keypad::Down,
                Keypad::Down,
                Keypad::Activate,
            ],
            (Code::Seven, Code::One) => vec![Keypad::Down, Keypad::Down, Keypad::Activate],
            (Code::Seven, Code::Two) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Right, Keypad::Activate]
            }
            (Code::Seven, Code::Three) => vec![
                Keypad::Down,
                Keypad::Down,
                Keypad::Right,
                Keypad::Right,
                Keypad::Activate,
            ],
            (Code::Seven, Code::Four) => vec![Keypad::Down, Keypad::Activate],
            (Code::Seven, Code::Five) => vec![Keypad::Down, Keypad::Right, Keypad::Activate],
            (Code::Seven, Code::Six) => {
                vec![Keypad::Down, Keypad::Right, Keypad::Right, Keypad::Activate]
            }
            (Code::Seven, Code::Eight) => vec![Keypad::Right, Keypad::Activate],
            (Code::Seven, Code::Nine) => vec![Keypad::Right, Keypad::Right, Keypad::Activate],
            (Code::Seven, Code::Activate) => vec![
                Keypad::Right,
                Keypad::Right,
                Keypad::Down,
                Keypad::Down,
                Keypad::Down,
                Keypad::Activate,
            ],
            (Code::Eight, Code::Zero) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Down, Keypad::Activate]
            }
            (Code::Eight, Code::One) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Left, Keypad::Activate]
            }
            (Code::Eight, Code::Two) => vec![Keypad::Down, Keypad::Down, Keypad::Activate],
            (Code::Eight, Code::Three) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Right, Keypad::Activate]
            }
            (Code::Eight, Code::Four) => vec![Keypad::Down, Keypad::Left, Keypad::Activate],
            (Code::Eight, Code::Five) => vec![Keypad::Down, Keypad::Activate],
            (Code::Eight, Code::Six) => vec![Keypad::Down, Keypad::Right, Keypad::Activate],
            (Code::Eight, Code::Seven) => vec![Keypad::Left, Keypad::Activate],
            (Code::Eight, Code::Nine) => vec![Keypad::Right, Keypad::Activate],
            (Code::Eight, Code::Activate) => vec![
                Keypad::Down,
                Keypad::Down,
                Keypad::Down,
                Keypad::Right,
                Keypad::Activate,
            ],
            (Code::Nine, Code::Zero) => vec![
                Keypad::Down,
                Keypad::Down,
                Keypad::Down,
                Keypad::Left,
                Keypad::Activate,
            ],
            (Code::Nine, Code::One) => vec![
                Keypad::Down,
                Keypad::Down,
                Keypad::Left,
                Keypad::Left,
                Keypad::Activate,
            ],
            (Code::Nine, Code::Two) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Left, Keypad::Activate]
            }
            (Code::Nine, Code::Three) => vec![Keypad::Down, Keypad::Down, Keypad::Activate],
            (Code::Nine, Code::Four) => {
                vec![Keypad::Down, Keypad::Left, Keypad::Left, Keypad::Activate]
            }
            (Code::Nine, Code::Five) => vec![Keypad::Down, Keypad::Left, Keypad::Activate],
            (Code::Nine, Code::Six) => vec![Keypad::Down, Keypad::Activate],
            (Code::Nine, Code::Seven) => vec![Keypad::Left, Keypad::Left, Keypad::Activate],
            (Code::Nine, Code::Eight) => vec![Keypad::Left, Keypad::Activate],
            (Code::Nine, Code::Activate) => {
                vec![Keypad::Down, Keypad::Down, Keypad::Down, Keypad::Activate]
            }
            (Code::Activate, Code::Zero) => vec![Keypad::Left, Keypad::Activate],
            (Code::Activate, Code::One) => {
                vec![Keypad::Up, Keypad::Left, Keypad::Left, Keypad::Activate]
            }
            (Code::Activate, Code::Two) => vec![Keypad::Left, Keypad::Up, Keypad::Activate],
            (Code::Activate, Code::Three) => vec![Keypad::Up, Keypad::Activate],
            (Code::Activate, Code::Four) => vec![
                Keypad::Up,
                Keypad::Up,
                Keypad::Left,
                Keypad::Left,
                Keypad::Activate,
            ],
            (Code::Activate, Code::Five) => {
                vec![Keypad::Left, Keypad::Up, Keypad::Up, Keypad::Activate]
            }
            (Code::Activate, Code::Six) => vec![Keypad::Up, Keypad::Up, Keypad::Activate],
            (Code::Activate, Code::Seven) => vec![
                Keypad::Up,
                Keypad::Up,
                Keypad::Up,
                Keypad::Left,
                Keypad::Left,
                Keypad::Activate,
            ],
            (Code::Activate, Code::Eight) => vec![
                Keypad::Up,
                Keypad::Up,
                Keypad::Up,
                Keypad::Left,
                Keypad::Activate,
            ],
            (Code::Activate, Code::Nine) => {
                vec![Keypad::Up, Keypad::Up, Keypad::Up, Keypad::Activate]
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Keypad {
    Up,
    Down,
    Left,
    Right,
    Activate,
}

impl Keypad {
    fn get_shortest_sequence(&self, destination: &Self) -> Vec<Keypad> {
        match (self, destination) {
            (x, y) if x == y => vec![Keypad::Activate],
            (Keypad::Up, Keypad::Down) => vec![Keypad::Down, Keypad::Activate],
            (Keypad::Up, Keypad::Left) => vec![Keypad::Down, Keypad::Left, Keypad::Activate],
            (Keypad::Up, Keypad::Right) => vec![Keypad::Down, Keypad::Right, Keypad::Activate],
            (Keypad::Up, Keypad::Activate) => vec![Keypad::Right, Keypad::Activate],
            (Keypad::Down, Keypad::Up) => vec![Keypad::Up, Keypad::Activate],
            (Keypad::Down, Keypad::Left) => vec![Keypad::Left, Keypad::Activate],
            (Keypad::Down, Keypad::Right) => vec![Keypad::Right, Keypad::Activate],
            (Keypad::Down, Keypad::Activate) => vec![Keypad::Up, Keypad::Right, Keypad::Activate],
            (Keypad::Left, Keypad::Up) => vec![Keypad::Right, Keypad::Up, Keypad::Activate],
            (Keypad::Left, Keypad::Down) => vec![Keypad::Right, Keypad::Activate],
            (Keypad::Left, Keypad::Right) => vec![Keypad::Right, Keypad::Right, Keypad::Activate],
            (Keypad::Left, Keypad::Activate) => {
                vec![Keypad::Right, Keypad::Right, Keypad::Up, Keypad::Activate]
            }
            (Keypad::Right, Keypad::Up) => vec![Keypad::Left, Keypad::Up, Keypad::Activate],
            (Keypad::Right, Keypad::Down) => vec![Keypad::Left, Keypad::Activate],
            (Keypad::Right, Keypad::Left) => vec![Keypad::Left, Keypad::Left, Keypad::Activate],
            (Keypad::Right, Keypad::Activate) => vec![Keypad::Up, Keypad::Activate],
            (Keypad::Activate, Keypad::Up) => vec![Keypad::Left, Keypad::Activate],
            (Keypad::Activate, Keypad::Down) => vec![Keypad::Left, Keypad::Down, Keypad::Activate],
            (Keypad::Activate, Keypad::Left) => {
                vec![Keypad::Down, Keypad::Left, Keypad::Left, Keypad::Activate]
            }
            (Keypad::Activate, Keypad::Right) => vec![Keypad::Down, Keypad::Activate],
            _ => unreachable!(),
        }
    }
}

#[problem_day]
fn run(Day21(input): Day21, arguments: &CommandLineArguments) -> usize {
    input
        .into_iter()
        .map(|code| {
            let button_presses = once(Code::Activate)
                .chain(code.clone())
                .tuple_windows()
                .map(|(source, dest)| {
                    press_button(
                        source.get_shortest_sequence(&dest),
                        arguments.n + 1,
                        &mut AHashMap::new(),
                    )
                })
                .sum::<usize>();

            button_presses
                    * code
                        .iter()
                        .filter(|item| !matches!(item, Code::Activate))
                        .rev()
                        .enumerate()
                        .map(|(index, code)| match code {
                            Code::Zero => 0,
                            Code::One => 1,
                            Code::Two => 2,
                            Code::Three => 3,
                            Code::Four => 4,
                            Code::Five => 5,
                            Code::Six => 6,
                            Code::Seven => 7,
                            Code::Eight => 8,
                            Code::Nine => 9,
                            Code::Activate => unreachable!(),
                        } * 10_usize.pow(index as u32))
                        .sum::<usize>()
        })
        .sum()
}

fn press_button(
    target_location: Vec<Keypad>,
    depth: usize,
    cache: &mut AHashMap<(Vec<Keypad>, usize), usize>,
) -> usize {
    if depth == 0 {
        return 1;
    }

    if let Some(result) = cache.get(&(target_location.clone(), depth)) {
        return *result;
    }

    let result = once(Keypad::Activate)
        .chain(target_location.clone())
        .tuple_windows()
        .map(|(source, dest)| press_button(source.get_shortest_sequence(&dest), depth - 1, cache))
        .sum();

    cache.insert((target_location, depth), result);

    result
}
