use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{breadth_first_search, BreadthFirstSearchLifecycle, PlanarCoordinate},
    parse::{parse_digit, parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use ahash::AHashSet;
use chumsky::{error::Rich, extra, Parser};
use clap::{Args, ValueEnum};
use either::Either;
use ndarray::Array2;
use std::{collections::VecDeque, iter, sync::LazyLock};

pub static DAY_10: LazyLock<CliProblem<Day10, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day10",
            "Scores various trailheads on a mountain",
            "Table of relative elevations for a mountain",
        )
        .with_part(
            "Scores a trailhead based on the number of unique peaks it leads to.",
            CommandLineArguments {
                scoring: ScoringSystem::UniquePeaks,
            },
            vec![("sample.txt", 1), ("sample2.txt", 36)],
        )
        .with_part(
            "Scores a trailhead based on the number of unique paths it has.",
            CommandLineArguments {
                scoring: ScoringSystem::UniquePaths,
            },
            vec![("sample2.txt", 81)],
        )
        .freeze()
    });

#[derive(ValueEnum, Clone)]
enum ScoringSystem {
    UniquePeaks,
    UniquePaths,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "Way to score a trail head")]
    scoring: ScoringSystem,
}

pub struct Day10(Array2<u32>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day10, extra::Err<Rich<'a, char>>> {
    parse_table2(parse_digit().map(|c| c.to_digit(10).expect("Works")))
        .map(Day10)
        .end()
}

#[problem_day]
fn run(Day10(input): Day10, arguments: &CommandLineArguments) -> usize {
    match arguments.scoring {
        ScoringSystem::UniquePeaks => find_trail_path_score(
            &input,
            |point| AHashSet::from([*point]),
            |point, peaks, score| {
                let trail_endings = score.get_mut(*point).expect("exists");
                trail_endings.extend(peaks.clone());
            },
            |score| score.len(),
        ),
        ScoringSystem::UniquePaths => find_trail_path_score(
            &input,
            |_point| 1,
            |point, peaks, score| *score.get_mut(*point).expect("exists") += peaks,
            |score| *score,
        ),
    }
}

fn find_trail_path_score<T: Clone + Default, F, G, H>(
    mountain: &Array2<u32>,
    init_score: F,
    mut add_to_score: G,
    collect_score: H,
) -> usize
where
    F: Fn(&(usize, usize)) -> T,
    G: FnMut(&(usize, usize), &T, &mut Array2<T>),
    H: Fn(&T) -> usize,
{
    let mut score = Array2::from_elem(mountain.dim(), T::default());
    let queue: VecDeque<(usize, usize)> = mountain
        .indexed_iter()
        .filter(|(_, value)| **value == 9)
        .map(|(index, _)| index)
        .inspect(|top| {
            *score.get_mut(*top).expect("Exists") = init_score(top);
        })
        .collect();

    let mut trail_heads = AHashSet::new();
    let mut visited = Array2::from_elem(mountain.dim(), false);

    breadth_first_search(
        queue,
        &mut visited,
        &mut BreadthFirstSearchLifecycle::get_adjacent::<()>(|location| {
            let height = mountain.get(*location).expect("Valid Index");
            if *height == 0 {
                trail_heads.insert(*location);
                return Either::Left(iter::empty::<(usize, usize)>());
            }

            Either::Right(
                location
                    .into_iter_cardinal_adjacent()
                    .filter(move |adjacent| {
                        mountain
                            .get(*adjacent)
                            .is_some_and(|position| *position == height - 1)
                    }),
            )
        })
        .with_on_insert(|location, valid_step| {
            let peaks = score.get(*location).expect("Exists").clone();
            add_to_score(valid_step, &peaks, &mut score);
        }),
    );

    trail_heads
        .into_iter()
        .map(|trail_head| collect_score(score.get(trail_head).expect("Exists")))
        .sum()
}
