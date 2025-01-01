use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{BoundedPoint, CardinalDirection, Direction, CARDINAL_DIRECTIONS},
    parse::{parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    Parser,
};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use ndarray::{Array2, Array3, Axis};
use std::sync::LazyLock;

pub static DAY_06: LazyLock<CliProblem<Day06, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day06",
            "Finds the best way to avoid the guard",
            "Floor plan with guard location and obstructions.",
        )
        .with_part(
            "Computes the total number of tiles the guard walks on.",
            CommandLineArguments {
                avoidence_strategy: AvoidenceStrategy::FullPath,
            },
            vec![("sample.txt", 41)],
        )
        .with_part(
            "Computes the number of possible loops when adding one obstruction.",
            CommandLineArguments {
                avoidence_strategy: AvoidenceStrategy::Loop,
            },
            vec![("sample.txt", 6)],
        )
        .freeze()
    });

#[derive(ValueEnum, Clone)]
enum AvoidenceStrategy {
    FullPath,
    Loop,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "The interpretation of the lists")]
    avoidence_strategy: AvoidenceStrategy,
}

pub struct Day06(Array2<Lab>);

#[derive(Clone)]
enum Lab {
    Open,
    Obstruction,
    Guard,
}

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day06, extra::Err<Rich<'a, char>>> {
    let open = just(".").to(Lab::Open);
    let obstruction = just("#").to(Lab::Obstruction);
    let guard = just("^").to(Lab::Guard);

    parse_table2(choice((open, obstruction, guard)))
        .map(Day06)
        .end()
}

#[problem_day]
fn run(Day06(input): Day06, arguments: &CommandLineArguments) -> usize {
    let (max_x, max_y) = BoundedPoint::maxes_from_table(&input);

    let guard_position = input
        .indexed_iter()
        .find(|(_, location)| matches!(location, Lab::Guard))
        .map(|(location, _)| BoundedPoint::from_table_index(location, max_x, max_y))
        .expect("Guard exists");
    let guard_facing = CardinalDirection::Up;

    let guard_path = run(guard_position, guard_facing, &input)
        .map(|visited| {
            visited
                .fold_axis(Axis(2), false, |acc, value| *acc || *value)
                .indexed_iter()
                .filter(|(_, value)| **value)
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
        })
        .expect("Result exists");

    match arguments.avoidence_strategy {
        AvoidenceStrategy::FullPath => guard_path.len(),
        AvoidenceStrategy::Loop => {
            let mut sparse_lab = build_obstruction_mapping(&input);
            let mut visited = Array3::from_elem((max_y + 1, max_x + 1, 4), 0);
            guard_path
                .into_iter()
                .map(|index| BoundedPoint::from_table_index(index, max_x, max_y))
                .filter(|point| *point != guard_position)
                .enumerate()
                .filter(|(index, obstruction)| {
                    let old = add_obstruction(*obstruction, &mut sparse_lab);
                    let result = does_guard_loop(
                        guard_position,
                        guard_facing,
                        (index + 1) as u16,
                        &sparse_lab,
                        &mut visited,
                    );
                    restore_lab(*obstruction, old, &mut sparse_lab);
                    result
                })
                .count()
        }
    }
}

fn direction_to_index(direction: &CardinalDirection) -> usize {
    match direction {
        CardinalDirection::Up => 0,
        CardinalDirection::Down => 1,
        CardinalDirection::Left => 2,
        CardinalDirection::Right => 3,
    }
}

fn run(
    mut guard_position: BoundedPoint,
    mut guard_facing: CardinalDirection,
    lab: &Array2<Lab>,
) -> Option<Array3<bool>> {
    let mut visited = Array3::from_elem(
        (guard_position.max_y + 1, guard_position.max_x + 1, 4),
        false,
    );

    *visited
        .get_mut((
            guard_position.y,
            guard_position.x,
            direction_to_index(&guard_facing),
        ))
        .expect("exists") = true;

    while let Some((direction, position)) = run_step(&guard_facing, &guard_position, lab) {
        let visit = visited
            .get_mut((position.y, position.x, direction_to_index(&direction)))
            .expect("Exists");
        if *visit {
            return None;
        }
        *visit = true;

        guard_facing = direction;
        guard_position = position;
    }

    Some(visited)
}

fn run_step(
    guard_facing: &CardinalDirection,
    guard_position: &BoundedPoint,
    lab: &Array2<Lab>,
) -> Option<(CardinalDirection, BoundedPoint)> {
    guard_position.get_adjacent(*guard_facing).map(|position| {
        match position.get_from_table(lab).expect("Valid position") {
            Lab::Obstruction => (guard_facing.get_clockwise(), *guard_position),
            _ => (*guard_facing, position),
        }
    })
}

fn build_obstruction_mapping(lab: &Array2<Lab>) -> Array2<Option<[Option<u8>; 4]>> {
    let mut lab_map: Array2<Option<[Option<u8>; 4]>> = Array2::from_shape_vec(
        lab.dim(),
        lab.rows()
            .into_iter()
            .flat_map(|row| {
                let row_chunks = row.into_iter().chunk_by(|item| match item {
                    Lab::Open | Lab::Guard => true,
                    Lab::Obstruction => false,
                });

                let mut previous_exists = false;
                let mut acc = Vec::new();
                let mut row_iter = row_chunks.into_iter().peekable();

                while let Some((is_open, chunk)) = row_iter.next() {
                    let chunk = chunk.collect::<Vec<_>>();

                    if !is_open {
                        for _ in 0..chunk.len() {
                            acc.push(None);
                        }

                        previous_exists = true;
                        continue;
                    }

                    let chunk_length = chunk.len();
                    let next_exists = row_iter.peek().is_some();

                    chunk.into_iter().enumerate().for_each(|(index, _)| {
                        let left = previous_exists.then_some(index as u8);
                        let right = next_exists.then(|| (chunk_length - index - 1) as u8);
                        acc.push(Some([None, None, left, right]));
                    });
                }
                acc
            })
            .collect::<Vec<_>>(),
    )
    .expect("Valid shape");

    lab_map.columns_mut().into_iter().for_each(|column| {
        let column_chunks = column.into_iter().chunk_by(|item| item.is_some());

        let mut previous_exists = false;
        let mut row_iter = column_chunks.into_iter().peekable();

        while let Some((is_open, chunk)) = row_iter.next() {
            if !is_open {
                previous_exists = true;
                continue;
            }

            let chunk = chunk.collect::<Vec<_>>();
            let chunk_length = chunk.len();
            let next_exists = row_iter.peek().is_some();

            chunk.into_iter().enumerate().for_each(|(index, value)| {
                let up = previous_exists.then_some(index as u8);
                let down = next_exists.then(|| (chunk_length - index - 1) as u8);
                value.iter_mut().for_each(|contents| {
                    contents[0] = up;
                    contents[1] = down;
                });
            });
        }
    });

    lab_map
}

fn add_obstruction(
    position: BoundedPoint,
    lab: &mut Array2<Option<[Option<u8>; 4]>>,
) -> [Option<u8>; 4] {
    let old = *position.get_from_table(lab).expect("exists");
    position.insert_into_table(None, lab);
    CARDINAL_DIRECTIONS.iter().for_each(|direction| {
        position
            .into_iter_direction(*direction)
            .enumerate()
            .take_while(|(index, point)| {
                lab.get_mut((point.y, point.x))
                    .expect("exists")
                    .iter_mut()
                    .for_each(|contents| {
                        contents[direction_to_index(&direction.get_opposite())] =
                            Some(*index as u8);
                    });

                point.get_from_table(lab).expect("exists").is_some()
            })
            .for_each(|_| ())
    });
    old.expect("Not an obstical already")
}

fn restore_lab(
    position: BoundedPoint,
    old: [Option<u8>; 4],
    lab: &mut Array2<Option<[Option<u8>; 4]>>,
) {
    position.insert_into_table(Some(old), lab);
    CARDINAL_DIRECTIONS.iter().for_each(|direction| {
        let offset = old[direction_to_index(&direction.get_opposite())];
        position
            .into_iter_direction(*direction)
            .enumerate()
            .take_while(|(index, point)| {
                lab.get_mut((point.y, point.x))
                    .expect("exists")
                    .iter_mut()
                    .for_each(|contents| {
                        contents[direction_to_index(&direction.get_opposite())] =
                            offset.map(|distance| distance + 1 + *index as u8);
                    });

                point.get_from_table(lab).expect("exists").is_some()
            })
            .for_each(|_| ())
    });
}

fn does_guard_loop(
    mut guard_position: BoundedPoint,
    mut guard_facing: CardinalDirection,
    visited_generation: u16,
    lab: &Array2<Option<[Option<u8>; 4]>>,
    visited: &mut Array3<u16>,
) -> bool {
    *visited
        .get_mut((
            guard_position.y,
            guard_position.x,
            direction_to_index(&guard_facing),
        ))
        .expect("exists") = visited_generation;

    while let Some((direction, position)) = run_step_sparse(&guard_facing, &guard_position, lab) {
        let visit = visited
            .get_mut((position.y, position.x, direction_to_index(&direction)))
            .expect("Exists");
        if *visit == visited_generation {
            return true;
        }
        *visit = visited_generation;

        guard_facing = direction;
        guard_position = position;
    }

    false
}

fn run_step_sparse(
    guard_facing: &CardinalDirection,
    guard_position: &BoundedPoint,
    lab: &Array2<Option<[Option<u8>; 4]>>,
) -> Option<(CardinalDirection, BoundedPoint)> {
    guard_position.get_from_table(lab).and_then(|location| {
        location
            .and_then(|indices| indices[direction_to_index(guard_facing)])
            .and_then(|distance| guard_position.stride_to(distance as usize, *guard_facing))
            .map(|next_position| (guard_facing.get_clockwise(), next_position))
    })
}
