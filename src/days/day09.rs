use crate::libs::{
    cli::{CliProblem, Freeze, new_cli_problem},
    parse::{StringParse, parse_digit},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{IterParser, Parser, error::Rich, extra, prelude::end, text};
use clap::{Args, ValueEnum};
use priority_queue::PriorityQueue;
use std::{
    array,
    cmp::{Reverse, min},
    hash::Hash,
    iter::repeat_n,
    sync::LazyLock,
};

pub static DAY_09: LazyLock<CliProblem<Day09, CommandLineArguments, Freeze>> =
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

pub struct Day09(Vec<DiskSection>);

#[derive(Debug)]
struct DiskSection {
    file_length: usize,
    free_length: usize,
}

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day09, extra::Err<Rich<'a, char>>> {
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
        .map(Day09)
}

#[problem_day]
fn run(Day09(input): Day09, arguments: &CommandLineArguments) -> usize {
    match arguments.compression_strategy {
        CompressionStrategy::HighestCompression => {
            let ids_with_files: Vec<_> = input.into_iter().enumerate().collect();
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

                let right_file_remaining = right_disk_section.file_length - right_used;
                let left_space_remaining = left_disk_section.free_length - left_space_used;

                match right_file_remaining.cmp(&left_space_remaining) {
                    std::cmp::Ordering::Less => {
                        file_system.append(&mut vec![right_file_id; right_file_remaining]);

                        left_space_used += right_file_remaining;
                        right_used = 0;
                        right -= 1;
                    }
                    std::cmp::Ordering::Greater => {
                        file_system.append(&mut vec![right_file_id; left_space_remaining]);

                        right_used += left_space_remaining;
                        left_space_used = 0;
                        left += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        file_system.append(&mut vec![right_file_id; left_space_remaining]);

                        right_used = 0;
                        left_space_used = 0;
                        right -= 1;
                        left += 1;
                    }
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
        CompressionStrategy::FirstAvailableSlot => compress_to_first_avilable_slot(&input),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Data {
    FileData(usize),
    Free,
}

#[derive(Debug)]
struct Block {
    block_id: usize,
    free_early: usize,
    allocated: Vec<(usize, usize)>,
    free: usize,
}

impl Eq for Block {}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.block_id == other.block_id
    }
}

impl Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block_id.hash(state);
    }
}

fn compress_to_first_avilable_slot(disk: &[DiskSection]) -> usize {
    let mut space_queues: [PriorityQueue<Block, Reverse<usize>>; 10] =
        array::from_fn(|_| PriorityQueue::new());
    let ids_with_files: Vec<_> = disk.iter().enumerate().collect();
    ids_with_files
        .iter()
        .map(|(id, file)| Block {
            block_id: *id,
            free_early: 0,
            allocated: vec![(*id, file.file_length)],
            free: file.free_length,
        })
        .for_each(|block| {
            let block_bucket = min(block.free, 9);
            let id = block.block_id;
            space_queues
                .get_mut(block_bucket)
                .expect("Exists")
                .push(block, Reverse(id));
        });

    let mut dummy_block = Block {
        block_id: 0,
        free_early: 0,
        allocated: Vec::new(),
        free: 0,
    };

    ids_with_files.iter().rev().for_each(|(id, file)| {
        if let Some(bucket) = (file.file_length..=9)
            .filter_map(|bucket| {
                space_queues
                    .get(bucket)
                    .and_then(|queue| queue.peek().map(|(_, index)| (*index, bucket)))
            })
            .max()
            .filter(|(index, _)| index.0 < *id)
            .map(|(_, bucket)| bucket)
        {
            let (mut new_block, new_index) = space_queues
                .get_mut(bucket)
                .and_then(|queue| queue.pop())
                .expect("Exists");

            // Update the new Block
            new_block.allocated.push((*id, file.file_length));
            new_block.free -= file.file_length;
            let block_bucket = min(new_block.free, 9);
            space_queues
                .get_mut(block_bucket)
                .expect("Exists")
                .push(new_block, new_index);

            // Update the old block
            dummy_block.block_id = *id;

            space_queues
                .iter_mut()
                .find(|queue| queue.get_priority(&dummy_block).is_some())
                .and_then(|queue| queue.remove(&dummy_block))
                .into_iter()
                .for_each(|(mut block, _)| {
                    block.allocated.retain(|(current, _)| id != current);
                    block.free_early += file.file_length;
                    let block_bucket = min(block.free, 9);
                    let block_id = block.block_id;
                    space_queues
                        .get_mut(block_bucket)
                        .expect("Exists")
                        .push(block, Reverse(block_id));
                });
        }
    });

    space_queues
        .into_iter()
        .flat_map(|queue| queue.into_iter())
        .fold(PriorityQueue::new(), |mut acc, (block, index)| {
            acc.push(block, index);
            acc
        })
        .into_sorted_iter()
        .flat_map(|(block, _)| {
            repeat_n(Data::Free, block.free_early)
                .chain(
                    block
                        .allocated
                        .into_iter()
                        .flat_map(|(id, length)| repeat_n(Data::FileData(id), length)),
                )
                .chain(repeat_n(Data::Free, block.free))
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
