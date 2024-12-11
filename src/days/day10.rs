use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::BoundedPoint,
    parse::{parse_digit, parse_table2, StringParse},
    problem::Problem,
};
use chumsky::{container::Container, error::Rich, extra, Parser};
use clap::{Args, ValueEnum};
use ndarray::Array2;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::LazyLock,
};

pub static DAY_10: LazyLock<CliProblem<Input, CommandLineArguments, Day10, Freeze>> =
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

pub struct Input(Array2<u32>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_table2(parse_digit().map(|c| c.to_digit(10).expect("Works"))).map(Input)
    }
}

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

pub struct Day10 {}

impl Problem<Input, CommandLineArguments> for Day10 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        match arguments.scoring {
            ScoringSystem::UniquePeaks => find_trail_path_score(
                &input.0,
                |point| HashSet::from([*point]),
                |point, peaks, score| {
                    let trail_endings = score.entry(*point).or_default();
                    trail_endings.extend(peaks.clone());
                },
                |score| score.len(),
            ),
            ScoringSystem::UniquePaths => find_trail_path_score(
                &input.0,
                |_point| 1,
                |point, peaks, score| *score.entry(*point).or_insert(0) += peaks,
                |score| *score,
            ),
        }
    }
}

fn find_trail_path_score<T: Clone, F, G, H>(
    mountain: &Array2<u32>,
    init_score: F,
    mut add_to_score: G,
    collect_score: H,
) -> usize
where
    F: Fn(&BoundedPoint) -> T,
    G: FnMut(&BoundedPoint, &T, &mut HashMap<BoundedPoint, T>),
    H: Fn(&T) -> usize,
{
    let (max_x, max_y) = BoundedPoint::maxes_from_table(mountain);

    let mut score = HashMap::new();
    let mut queue: VecDeque<BoundedPoint> = mountain
        .indexed_iter()
        .filter(|(_, value)| **value == 9)
        .map(|(index, _)| BoundedPoint::from_table_index(index, max_x, max_y))
        .inspect(|top| {
            score.insert(*top, init_score(top));
        })
        .collect();

    let mut trail_heads = HashSet::new();
    let mut visited = HashSet::new();

    while let Some(location) = queue.pop_front() {
        if visited.contains(&location) {
            continue;
        }

        visited.push(location);

        let height = location.get_from_table(mountain).expect("Valid index");
        if *height == 0 {
            trail_heads.insert(location);
            continue;
        }
        let peaks = score.get(&location).expect("Exists").clone();

        location
            .into_iter_cardinal_adjacent()
            .filter(|adjacent| *adjacent.get_from_table(mountain).expect("exists") == height - 1)
            .for_each(|valid_step| {
                add_to_score(&valid_step, &peaks, &mut score);
                queue.push_back(valid_step);
            });
    }

    trail_heads
        .into_iter()
        .map(|trail_head| collect_score(score.get(&trail_head).expect("Exists")))
        .sum()
}
