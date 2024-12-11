use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_digit, StringParse},
    problem::Problem,
};
use chumsky::{error::Rich, extra, prelude::end, text, IterParser, Parser};
use clap::{Args, ValueEnum};
use std::{iter::repeat, sync::LazyLock};

pub static DAY_09: LazyLock<CliProblem<Input, CommandLineArguments, Day09, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day09",
            "Moves files around in a file system to get more space",
            "Contiguous list of file sizes followed by the free space after the file.",
        )
        .with_part(
            "Files should move left in the file system",
            CommandLineArguments {
                compression_strategy: CompressionStrategy::HighestCompression,
            },
            vec![("sample.txt", 1928)],
        )
        .with_part(
            "Move a file left into the first available slot that fits it.",
            CommandLineArguments {
                compression_strategy: CompressionStrategy::FirstAvailableSlot,
            },
            vec![("sample.txt", 2858)],
        )
        .freeze()
    });

pub struct Input(Vec<DiskSection>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let disk_section = parse_digit()
            .map(|c| c.to_digit(10).expect("Works"))
            .then(
                parse_digit()
                    .map(|c| c.to_digit(10).expect("Works"))
                    .or(text::newline().to(0)),
            )
            .map(|(file_length, free_length)| DiskSection {
                file_length: file_length as usize,
                free_length: free_length as usize,
            });
        disk_section
            .repeated()
            .collect()
            .then_ignore(end())
            .map(Input)
    }
}

#[derive(Debug)]
struct DiskSection {
    file_length: usize,
    free_length: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Data {
    FileData(usize),
    Free,
}

#[derive(ValueEnum, Clone)]
enum CompressionStrategy {
    HighestCompression,
    FirstAvailableSlot,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "Strategy to reformat the memory")]
    compression_strategy: CompressionStrategy,
}

pub struct Day09 {}

impl Problem<Input, CommandLineArguments> for Day09 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let mut ids_with_files: Vec<_> = input.0.into_iter().enumerate().collect();

        match arguments.compression_strategy {
            CompressionStrategy::HighestCompression => {
                let mut left = 0;
                let mut right = ids_with_files.len() - 1;
                let mut right_used = 0;
                let mut left_space_used = 0;
                let mut file_system = Vec::new();

                while left < right {
                    let (left_file_id, left_disk_section) = &ids_with_files[left];
                    let (right_file_id, right_disk_section) = &ids_with_files[right];

                    if left_space_used == 0 {
                        file_system.append(&mut vec![left_file_id; left_disk_section.file_length]);
                    }

                    if (right_disk_section.file_length - right_used)
                        < (left_disk_section.free_length - left_space_used)
                    {
                        file_system.append(&mut vec![
                            right_file_id;
                            right_disk_section.file_length - right_used
                        ]);

                        left_space_used += right_disk_section.file_length - right_used;
                        right_used = 0;
                        right -= 1;
                    } else if (right_disk_section.file_length - right_used)
                        > (left_disk_section.free_length - left_space_used)
                    {
                        file_system.append(&mut vec![
                            right_file_id;
                            left_disk_section.free_length
                                - left_space_used
                        ]);

                        right_used += left_disk_section.free_length - left_space_used;
                        left_space_used = 0;
                        left += 1;
                    } else {
                        file_system.append(&mut vec![
                            right_file_id;
                            left_disk_section.free_length
                                - left_space_used
                        ]);

                        right_used = 0;
                        left_space_used = 0;
                        right -= 1;
                        left += 1;
                    }
                }

                let (right_file_id, right_disk_section) = &ids_with_files[right];
                file_system.append(&mut vec![
                    right_file_id;
                    right_disk_section.file_length - right_used
                ]);

                file_system
                    .into_iter()
                    .enumerate()
                    .map(|(index, id)| index * id)
                    .sum()
            }
            CompressionStrategy::FirstAvailableSlot => {
                let mut right = ids_with_files.len() - 1;

                loop {
                    let (right_file_id, right_file_length, right_free_length) = {
                        let (right_file_id, right_disk_section) = &ids_with_files[right];
                        (
                            *right_file_id,
                            right_disk_section.file_length,
                            right_disk_section.free_length,
                        )
                    };

                    let new_position = ids_with_files
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index < right)
                        .find(|(_, (_, section))| section.free_length >= right_file_length)
                        .map(|(index, _)| index)
                        .unwrap_or(right);

                    if new_position == right {
                        if right == 0 {
                            break;
                        }

                        right -= 1;
                    } else {
                        let (_, adjacent_file) = ids_with_files.get_mut(right - 1).expect("exists");
                        adjacent_file.free_length += right_file_length + right_free_length;

                        let (_, previous_file) =
                            ids_with_files.get_mut(new_position).expect("exists");
                        let new_free = previous_file.free_length - right_file_length;
                        previous_file.free_length = 0;
                        let new_disk = (
                            right_file_id,
                            DiskSection {
                                file_length: right_file_length,
                                free_length: new_free,
                            },
                        );

                        ids_with_files.remove(right);
                        ids_with_files.insert(new_position + 1, new_disk);
                    }
                }

                ids_with_files
                    .into_iter()
                    .flat_map(|(id, file)| {
                        repeat(Data::FileData(id))
                            .take(file.file_length)
                            .chain(repeat(Data::Free).take(file.free_length))
                    })
                    .enumerate()
                    .map(|(index, id)| {
                        index
                            * match id {
                                Data::FileData(id) => id,
                                Data::Free => 0,
                            }
                    })
                    .sum()
            }
        }
    }
}
