use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{
        BoundedPoint, CardinalDirection, HorizontalDirection, VerticalDirection,
        CARDINAL_DIRECTIONS,
    },
    parse::{parse_table2, StringParse},
    problem::Problem,
};
use chumsky::{error::Rich, extra, prelude::one_of, Parser};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use ndarray::Array2;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::LazyLock,
};

pub static DAY_12: LazyLock<CliProblem<Input, CommandLineArguments, Day12, Freeze>> =
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

pub struct Input(Array2<char>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_table2(one_of('A'..='Z')).map(Input)
    }
}

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

pub struct Day12 {}

impl Problem<Input, CommandLineArguments> for Day12 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let (max_x, max_y) = BoundedPoint::maxes_from_table(&input.0);
        let mut visited = HashSet::new();
        let mut regions = Vec::new();

        for (index, plot) in input.0.indexed_iter() {
            let current = BoundedPoint::from_table_index(index, max_x, max_y);
            if visited.contains(&current) {
                continue;
            }

            let mut region = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(current);

            while let Some(next_plot) = queue.pop_front() {
                if visited.contains(&next_plot) {
                    continue;
                }

                visited.insert(next_plot);
                region.push(next_plot);

                next_plot
                    .into_iter_cardinal_adjacent()
                    .filter(|adjacent| !visited.contains(adjacent))
                    .filter(|adjacent| adjacent.get_from_table(&input.0).expect("Exists") == plot)
                    .for_each(|adjacent| queue.push_back(adjacent));
            }

            regions.push(region);
        }

        let fence_score = match arguments.fence_score {
            FenceScore::Perimeter => count_perimeter,
            FenceScore::Fences => count_fences,
        };

        regions
            .into_iter()
            .map(|region| score_region(&region, &input.0, fence_score))
            .sum()
    }
}

fn score_region<F>(region: &[BoundedPoint], field: &Array2<char>, fence_function: F) -> usize
where
    F: FnOnce(&[BoundedPoint], &Array2<char>) -> usize,
{
    let area = region.len();

    area * fence_function(region, field)
}

fn count_perimeter(region: &[BoundedPoint], field: &Array2<char>) -> usize {
    region
        .iter()
        .map(|point| {
            let plot = point.get_from_table(field).expect("Exists");
            CARDINAL_DIRECTIONS
                .iter()
                .filter(|direction| {
                    point
                        .get_adjacent(**direction)
                        .and_then(|adjacent| adjacent.get_from_table(field))
                        .is_none_or(|other_plot| plot != other_plot)
                })
                .count()
        })
        .sum()
}

fn count_fences(region: &[BoundedPoint], field: &Array2<char>) -> usize {
    let north_fences = count_vertical_fences(VerticalDirection::Up, region, field);
    let east_fences = count_horizontal_fences(HorizontalDirection::Right, region, field);
    let south_fences = count_vertical_fences(VerticalDirection::Down, region, field);
    let west_fences = count_horizontal_fences(HorizontalDirection::Left, region, field);
    north_fences + east_fences + south_fences + west_fences
}

fn count_horizontal_fences(
    direction: HorizontalDirection,
    region: &[BoundedPoint],
    field: &Array2<char>,
) -> usize {
    count_direction_fences(direction, region, field, |mut acc, (point, _)| {
        let row: &mut Vec<_> = acc.entry(point.x).or_default();
        row.push(point.y);
        acc
    })
}

fn count_vertical_fences(
    direction: VerticalDirection,
    region: &[BoundedPoint],
    field: &Array2<char>,
) -> usize {
    count_direction_fences(direction, region, field, |mut acc, (point, _)| {
        let row: &mut Vec<_> = acc.entry(point.y).or_default();
        row.push(point.x);
        acc
    })
}

fn count_direction_fences<F>(
    direction: impl Into<CardinalDirection> + Copy,
    region: &[BoundedPoint],
    field: &Array2<char>,
    fold: F,
) -> usize
where
    F: FnMut(HashMap<usize, Vec<usize>>, (&BoundedPoint, &char)) -> HashMap<usize, Vec<usize>>,
{
    region
        .iter()
        .map(|point| (point, point.get_from_table(field).expect("Exists")))
        .filter(|(point, plot)| {
            point
                .get_adjacent(direction.into())
                .and_then(|adjacent| adjacent.get_from_table(field))
                .is_none_or(|other_plot| *plot != other_plot)
        })
        .fold(HashMap::new(), fold)
        .values_mut()
        .map(|axis| {
            axis.sort();
            axis.iter()
                .coalesce(|before, after| {
                    if *before + 1 == *after {
                        Ok(after)
                    } else {
                        Err((before, after))
                    }
                })
                .count()
        })
        .sum()
}
