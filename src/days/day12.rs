use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{breadth_first_search, PlanarCoordinate, PointDirection, CARDINAL_DIRECTIONS},
    parse::{parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{error::Rich, extra, prelude::one_of, Parser};
use clap::{Args, ValueEnum};
use ndarray::Array2;
use std::{collections::VecDeque, sync::LazyLock};

pub static DAY_12: LazyLock<CliProblem<Day12, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day12",
            "Finds the total cost for fences around garden plots",
            "Table of garden plots, each letter represents a different type of plant",
        )
        .with_part(
            "Computes the sum where perimeter segments each counting for the cost",
            CommandLineArguments {
                fence_score: FenceScore::Perimeter,
            },
            vec![
                ("sample.txt", 140),
                ("sample2.txt", 772),
                ("sample3.txt", 1930),
            ],
        )
        .with_part(
            "Computes the sum where each full straight fence counts for the cost",
            CommandLineArguments {
                fence_score: FenceScore::Fences,
            },
            vec![
                ("sample.txt", 80),
                ("sample4.txt", 236),
                ("sample5.txt", 368),
                ("sample3.txt", 1206),
            ],
        )
        .freeze()
    });

#[derive(ValueEnum, Clone)]
enum FenceScore {
    Perimeter,
    Fences,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "How to score the fence prices for a region")]
    fence_score: FenceScore,
}

pub struct Day12(Array2<char>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day12, extra::Err<Rich<'a, char>>> {
    parse_table2(one_of('A'..='Z')).map(Day12).end()
}

#[problem_day]
fn run(Day12(input): Day12, arguments: &CommandLineArguments) -> usize {
    let mut visited = Array2::from_elem(input.dim(), false);
    let mut regions = Vec::new();

    for (current, plot) in input.indexed_iter() {
        if *visited.get(current).unwrap_or(&false) {
            continue;
        }

        let mut region = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(current);

        breadth_first_search(
            queue,
            &mut visited,
            |_| None,
            |next_plot| {
                region.push(*next_plot);
                None::<()>
            },
            |next_plot| {
                next_plot
                    .into_iter_cardinal_adjacent()
                    .filter(|adjacent| input.get(*adjacent).is_some_and(|other| other == plot))
            },
            |_, _| (),
        );

        regions.push(region);
    }

    let fence_score = match arguments.fence_score {
        FenceScore::Perimeter => count_perimeter,
        FenceScore::Fences => count_corners,
    };

    regions
        .into_iter()
        .map(|region| score_region(&region, &input, fence_score))
        .sum()
}

fn score_region<F>(region: &[(usize, usize)], field: &Array2<char>, fence_function: F) -> usize
where
    F: FnOnce(&[(usize, usize)], &Array2<char>) -> usize,
{
    let area = region.len();

    area * fence_function(region, field)
}

fn count_corners(region: &[(usize, usize)], field: &Array2<char>) -> usize {
    region
        .iter()
        .map(|point| {
            let plot = field.get(*point).expect("Exists");
            let left = point
                .get_adjacent(PointDirection::Left)
                .and_then(|adjacent| field.get(adjacent));
            let up = point
                .get_adjacent(PointDirection::Up)
                .and_then(|adjacent| field.get(adjacent));
            let up_left = point
                .get_adjacent(PointDirection::UpLeft)
                .and_then(|adjacent| field.get(adjacent));
            let up_right = point
                .get_adjacent(PointDirection::UpRight)
                .and_then(|adjacent| field.get(adjacent));
            let right: Option<&char> = point
                .get_adjacent(PointDirection::Right)
                .and_then(|adjacent| field.get(adjacent));
            let down_right = point
                .get_adjacent(PointDirection::DownRight)
                .and_then(|adjacent| field.get(adjacent));
            let down = point
                .get_adjacent(PointDirection::Down)
                .and_then(|adjacent| field.get(adjacent));
            let down_left = point
                .get_adjacent(PointDirection::DownLeft)
                .and_then(|adjacent| field.get(adjacent));

            let mut count = 0;

            if is_corner(plot, &up_left, &left, &up) {
                count += 1;
            }

            if is_corner(plot, &up_right, &right, &up) {
                count += 1;
            }

            if is_corner(plot, &down_left, &left, &down) {
                count += 1;
            }

            if is_corner(plot, &down_right, &right, &down) {
                count += 1;
            }

            count
        })
        .sum()
}

fn is_corner(
    plot: &char,
    diagnal: &Option<&char>,
    horizontal: &Option<&char>,
    vertical: &Option<&char>,
) -> bool {
    (vertical.is_some_and(|adjacent| adjacent == plot)
        && horizontal.is_some_and(|adjacent| adjacent == plot)
        && diagnal.is_none_or(|adjacent| adjacent != plot))
        || (vertical.is_none_or(|adjacent| adjacent != plot)
            && horizontal.is_none_or(|adjacent| adjacent != plot))
}

fn count_perimeter(region: &[(usize, usize)], field: &Array2<char>) -> usize {
    region
        .iter()
        .map(|point| {
            let plot = field.get(*point).expect("Exists");
            CARDINAL_DIRECTIONS
                .iter()
                .filter(|direction| {
                    point
                        .get_adjacent(**direction)
                        .and_then(|adjacent| field.get(adjacent))
                        .is_none_or(|other_plot| plot != other_plot)
                })
                .count()
        })
        .sum()
}
