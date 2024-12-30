use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_between_blank_lines, parse_isize, StringParse},
    problem::Problem,
};
use adventofcode_macro::problem_day;
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::Args;
use std::sync::LazyLock;

pub static DAY_13: LazyLock<CliProblem<Input, CommandLineArguments, Day13, Freeze>> = LazyLock::new(
    || {
        new_cli_problem(
            "day13",
            "Finds the minimum cost to win the prizes if possible",
            "How far each button takes you for each press, and the prize location. Separated by blank lines",
        )
        .with_part(
            "Finds the sum needed to get all obtainable prizes",
            CommandLineArguments { offset: 0 },
            vec![("sample.txt", 480)],
        )
        .with_part(
            "Finds the sum needed for all obtainable prizes with the position offset of 10_000_000_000_000 ",
            CommandLineArguments { offset: OFFSET },
            vec![],
        )
        .freeze()
    },
);

pub struct Input(Vec<Game>);

#[derive(Debug)]
struct Game {
    a: Button,
    b: Button,
    prize: (isize, isize),
}

#[derive(Debug)]
struct Button {
    dx: isize,
    dy: isize,
}

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let game = just("Button A: ")
            .ignore_then(parse_button())
            .then_ignore(just("\nButton B: "))
            .then(parse_button())
            .then_ignore(just("\nPrize: X="))
            .then(parse_isize())
            .then_ignore(just(", Y="))
            .then(parse_isize())
            .map(|(((a, b), c), d)| Game {
                a,
                b,
                prize: (c, d),
            });
        parse_between_blank_lines(game).map(Input)
    }
}

fn parse_button<'a>() -> impl Parser<'a, &'a str, Button, extra::Err<Rich<'a, char>>> {
    just("X+")
        .ignore_then(parse_isize())
        .then_ignore(just(", Y+"))
        .then(parse_isize())
        .map(|(dx, dy)| Button { dx, dy })
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "The offset to the prize's position")]
    offset: isize,
}

const OFFSET: isize = 10_000_000_000_000;

#[problem_day(Day13)]
fn run(input: Input, arguments: &CommandLineArguments) -> isize {
    input
        .0
        .into_iter()
        .flat_map(|game| {
            calculate_game_cost(
                &game.a,
                &game.b,
                game.prize.0 + arguments.offset,
                game.prize.1 + arguments.offset,
            )
        })
        .sum()
}

fn calculate_game_cost(a: &Button, b: &Button, tx: isize, ty: isize) -> Option<isize> {
    let divisor = b.dy * a.dx - a.dy * b.dx;
    if divisor == 0 {
        return None;
    }
    let a_numerator = tx * b.dy - ty * b.dx;
    let b_numerator = ty * a.dx - tx * a.dy;

    if a_numerator % divisor != 0 || b_numerator % divisor != 0 {
        return None;
    }

    let a_count = a_numerator / divisor;
    let b_count = b_numerator / divisor;

    Some(a_count * 3 + b_count)
}
