use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{PlanarCoordinate, PointDirection, RADIAL_DIRECTIONS},
    parse::{parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{error::Rich, extra, prelude::one_of, Parser};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use ndarray::Array2;
use std::sync::LazyLock;

pub static DAY_04: LazyLock<CliProblem<Day04, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day04",
            "Searches a word search for all instances",
            "Table of letters",
        )
        .with_part(
            "Counts all instances of XMAS for all directions",
            CommandLineArguments {
                search_setting: SearchSetting::Xmas,
            },
            vec![("sample.txt", 18)],
        )
        .with_part(
            "Counts all instances of a crossed MAS",
            CommandLineArguments {
                search_setting: SearchSetting::MasInX,
            },
            vec![("sample.txt", 9)],
        )
        .freeze()
    });

#[derive(ValueEnum, Clone)]
enum SearchSetting {
    Xmas,
    MasInX,
}

#[derive(Args)]
pub struct CommandLineArguments {
    search_setting: SearchSetting,
}

pub struct Day04(Array2<char>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day04, extra::Err<Rich<'a, char>>> {
    parse_table2(one_of("XMAS")).map(Day04).end()
}

#[problem_day]
fn run(Day04(input): Day04, arguments: &CommandLineArguments) -> usize {
    match arguments.search_setting {
        SearchSetting::Xmas => input
            .indexed_iter()
            .filter(|(_, value)| **value == 'X')
            .map(|(index, _)| index)
            .map(|point| number_of_xmas_from_point(&point, &input))
            .sum(),
        SearchSetting::MasInX => input
            .indexed_iter()
            .filter(|(_, value)| **value == 'A')
            .map(|(index, _)| index)
            .filter(|point| is_mas_from_point(point, &input))
            .count(),
    }
}

const DIAGNAL_1: [PointDirection; 2] = [PointDirection::UpRight, PointDirection::DownLeft];
const DIAGNAL_2: [PointDirection; 2] = [PointDirection::UpLeft, PointDirection::DownRight];
const DIAGNALS: [[PointDirection; 2]; 2] = [DIAGNAL_1, DIAGNAL_2];

fn is_mas_from_point(point: &(usize, usize), search: &Array2<char>) -> bool {
    DIAGNALS.into_iter().all(|diagnal| {
        diagnal
            .into_iter()
            .flat_map(|direction| point.get_adjacent(direction))
            .flat_map(|point| search.get(point))
            .fold((0, 0), |(mut m_count, mut s_count), c| {
                if *c == 'M' {
                    m_count += 1;
                } else if *c == 'S' {
                    s_count += 1;
                }

                (m_count, s_count)
            })
            == (1, 1)
    })
}

const MAS: [char; 3] = ['M', 'A', 'S'];

fn number_of_xmas_from_point(point: &(usize, usize), search: &Array2<char>) -> usize {
    RADIAL_DIRECTIONS
        .into_iter()
        .filter(|direction| {
            MAS.into_iter()
                .zip_longest(
                    point
                        .into_iter_direction(*direction)
                        .take(3)
                        .flat_map(|point| search.get(point)),
                )
                .all(|items| match items {
                    itertools::EitherOrBoth::Both(a, b) => a == *b,
                    _ => false,
                })
        })
        .count()
}
