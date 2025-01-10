use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{
        breadth_first_search, CardinalDirection, HorizontalDirection, PlanarCoordinate,
        VerticalDirection, CARDINAL_DIRECTIONS,
    },
    parse::{parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse, StringParse};
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    Parser,
};
use clap::Args;
use ndarray::Array2;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{collections::VecDeque, sync::LazyLock};

pub static DAY_20: LazyLock<CliProblem<Day20, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day20",
            "Finds the best cheating routes in a maze",
            "Table maze with only 1 valid path",
        )
        .with_part(
            "Computes the number of cheats that save over 100ns with a cheat value of 2",
            CommandLineArguments {
                cheat_threshold: 2,
                target_savings: 100,
                parallel: false,
            },
            vec![],
        )
        .with_part(
            "Computes the number of cheats that save over 100ns with a cheat value of 20",
            CommandLineArguments {
                cheat_threshold: 20,
                target_savings: 100,
                parallel: true,
            },
            vec![],
        )
        .freeze()
    });

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(
        short,
        long,
        help = "How much time you're allowed to cheat through walls."
    )]
    cheat_threshold: usize,

    #[arg(short, long, help = "How much time must be saved to bother cheating.")]
    target_savings: usize,

    #[arg(
        short,
        long,
        help = "Whether to run the cheat detection in parallel or not."
    )]
    parallel: bool,
}

#[derive(Debug, Clone, StringParse)]
enum Track {
    #[literal("S")]
    Start,
    #[literal("E")]
    End,
    #[literal(".")]
    Open,
    #[literal("#")]
    Wall,
}

pub struct Day20(Array2<Track>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day20, extra::Err<Rich<'a, char>>> {
    parse_table2(Track::parse()).map(Day20).end()
}

#[problem_day]
fn run(Day20(input): Day20, arguments: &CommandLineArguments) -> usize {
    let start = input
        .indexed_iter()
        .find(|(_, tile)| matches!(tile, Track::Start))
        .map(|(index, _)| index)
        .expect("Exists");
    let end = input
        .indexed_iter()
        .find(|(_, tile)| matches!(tile, Track::End))
        .map(|(index, _)| index)
        .expect("Exists");

    let path = shortest_path_full(&start, &end, &input);
    best_shortcuts(
        &end,
        arguments.cheat_threshold,
        arguments.target_savings,
        &path,
        arguments.parallel,
    )
}

fn generate_manhattan_quarter_points(distance: usize) -> Vec<(usize, usize)> {
    (1..=distance)
        .flat_map(|distance| {
            (0..distance)
                .map(|offset| {
                    let remainder = distance - offset;
                    (offset, remainder)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn best_shortcuts(
    end: &(usize, usize),
    cheat_threshold: usize,
    target_savings: usize,
    path: &Array2<Option<usize>>,
    parallel: bool,
) -> usize {
    let baseline = path.get(*end).expect("exists").expect("exists");
    let points = generate_manhattan_quarter_points(cheat_threshold);
    if parallel {
        path.indexed_iter()
            .par_bridge()
            .filter_map(|(tile, value)| value.filter(|_| tile != *end).map(|length| (tile, length)))
            .map(|(tile, length)| {
                worthy_cheats_from_tile(&tile, &length, &baseline, target_savings, &points, path)
            })
            .sum()
    } else {
        path.indexed_iter()
            .filter_map(|(tile, value)| value.filter(|_| tile != *end).map(|length| (tile, length)))
            .map(|(tile, length)| {
                worthy_cheats_from_tile(&tile, &length, &baseline, target_savings, &points, path)
            })
            .sum()
    }
}

fn worthy_cheats_from_tile(
    tile: &(usize, usize),
    length: &usize,
    baseline: &usize,
    target_savings: usize,
    points: &[(usize, usize)],
    path: &Array2<Option<usize>>,
) -> usize {
    CARDINAL_DIRECTIONS
        .iter()
        .map(|direction| {
            let (swap, veritcal, horizontal) = match direction {
                CardinalDirection::Up => (false, VerticalDirection::Up, HorizontalDirection::Right),
                CardinalDirection::Right => {
                    (true, VerticalDirection::Down, HorizontalDirection::Right)
                }
                CardinalDirection::Down => {
                    (false, VerticalDirection::Down, HorizontalDirection::Left)
                }
                CardinalDirection::Left => (true, VerticalDirection::Up, HorizontalDirection::Left),
            };
            points
                .iter()
                .filter_map(|(p1, p2)| {
                    let (p1, p2) = if swap { (*p2, *p1) } else { (*p1, *p2) };
                    tile.jump_to(p2, horizontal, p1, veritcal)
                        .map(|new_tile| (p1 + p2, new_tile))
                })
                .filter_map(|(distance, point)| {
                    path.get(point)
                        .and_then(|length| length.map(|length| (distance, length)))
                })
                .filter(|(_, other_length)| length < other_length)
                .map(|(distance, other_length)| {
                    let remaining_length = baseline - other_length;
                    length + distance + remaining_length
                })
                .map(|score| baseline - score)
                .filter(|savings| *savings >= target_savings)
                .count()
        })
        .sum::<usize>()
}

fn shortest_path_full(
    start: &(usize, usize),
    end: &(usize, usize),
    track: &Array2<Track>,
) -> Array2<Option<usize>> {
    let mut queue = VecDeque::new();
    let mut visited = Array2::from_elem(track.dim(), None);

    queue.push_back((*start, 0));

    breadth_first_search(
        queue,
        &mut visited,
        |(tile, _)| (tile == end).then_some(()),
        |(tile, length)| {
            let new_length = length + 1;
            tile.into_iter_cardinal_adjacent()
                .filter(|adjacent| {
                    matches!(
                        track.get(*adjacent).expect("exists"),
                        Track::End | Track::Start | Track::Open
                    )
                })
                .map(move |adjacent| (adjacent, new_length))
        },
        |_, _| (),
    );

    visited
}
