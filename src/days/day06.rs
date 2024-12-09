use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{BoundedPoint, CardinalDirection, Direction},
    parse::{parse_table2, StringParse},
    problem::Problem,
};
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    Parser,
};
use clap::{Args, ValueEnum};
use ndarray::{Array2, Array3, Axis};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::sync::LazyLock;

pub static DAY_06: LazyLock<CliProblem<Input, CommandLineArguments, Day06, Freeze>> =
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

pub struct Input(Array2<Lab>);

#[derive(Clone)]
enum Lab {
    Open,
    Obstruction,
    Guard,
}

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let open = just(".").to(Lab::Open);
        let obstruction = just("#").to(Lab::Obstruction);
        let guard = just("^").to(Lab::Guard);

        parse_table2(choice((open, obstruction, guard))).map(Input)
    }
}

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

pub struct Day06 {}

impl Problem<Input, CommandLineArguments> for Day06 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let (max_x, max_y) = BoundedPoint::maxes_from_table(&input.0);
        let guard_position = input
            .0
            .indexed_iter()
            .find(|(_, location)| matches!(location, Lab::Guard))
            .map(|(location, _)| BoundedPoint::from_table_index(location, max_x, max_y))
            .expect("Guard exists");
        let guard_facing = CardinalDirection::Up;

        let guard_path = run(guard_position, guard_facing, &input.0)
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
            AvoidenceStrategy::Loop => guard_path
                .into_iter()
                .map(|index| BoundedPoint::from_table_index(index, max_x, max_y))
                .filter(|point| *point != guard_position)
                .par_bridge()
                .map(|obstruction| add_obstruction(obstruction, input.0.clone()))
                .filter(|obstructed_lab| {
                    run(guard_position, guard_facing, obstructed_lab).is_none()
                })
                .count(),
        }
    }
}

fn add_obstruction(position: BoundedPoint, mut lab: Array2<Lab>) -> Array2<Lab> {
    position.insert_into_table(Lab::Obstruction, &mut lab);
    lab
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
        if *visited
            .get((position.y, position.x, direction_to_index(&direction)))
            .unwrap_or(&false)
        {
            return None;
        }
        *visited
            .get_mut((position.y, position.x, direction_to_index(&direction)))
            .expect("exists") = true;

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
