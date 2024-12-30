use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_usize, StringParse},
    problem::Problem,
};
use adventofcode_macro::problem_day;
use ahash::{AHashMap, AHashSet};
use chumsky::{
    error::Rich,
    extra,
    prelude::{end, just},
    text, IterParser, Parser,
};
use clap::Args;
use std::sync::LazyLock;

pub static DAY_05: LazyLock<CliProblem<Input, CommandLineArguments, Day05, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day05",
            "Returns the sum of the median valid page updates",
            "Newline delimited page rules followed by a newline delimited page update list",
        )
        .with_part(
            "Computes the sum of only valid page updates",
            CommandLineArguments { valid: true },
            vec![("sample.txt", 143)],
        )
        .with_part(
            "Computes the sum of the invalid updates once fixed",
            CommandLineArguments { valid: false },
            vec![("sample.txt", 123)],
        )
        .freeze()
    });

#[derive(Debug)]
pub struct Input {
    page_rules: Vec<(usize, usize)>,
    page_updates: Vec<Vec<usize>>,
}

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let page_rules = parse_usize()
            .then_ignore(just("|"))
            .then(parse_usize())
            .separated_by(text::newline())
            .at_least(1)
            .collect();
        let page_updates = parse_usize()
            .separated_by(just(","))
            .at_least(1)
            .collect()
            .separated_by(text::newline())
            .at_least(1)
            .collect();

        page_rules
            .then_ignore(text::newline().repeated().at_least(1))
            .then(page_updates)
            .then_ignore(text::newline().repeated())
            .then_ignore(end())
            .map(|(page_rules, page_updates)| Input {
                page_rules,
                page_updates,
            })
    }
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "If the updates should be valid or not")]
    valid: bool,
}

#[problem_day(Day05)]
fn run(input: Input, arguments: &CommandLineArguments) -> usize {
    let rule_map = build_page_rule_mapping(&input.page_rules);

    if arguments.valid {
        input
            .page_updates
            .into_iter()
            .filter(|page_update| is_valid_page_update(page_update, &rule_map))
            .map(|page_update| *page_update.get(page_update.len() / 2).unwrap_or(&0))
            .sum()
    } else {
        input
            .page_updates
            .into_iter()
            .filter(|page_update| !is_valid_page_update(page_update, &rule_map))
            .map(|page_update| find_center_of_page_update(&page_update, &rule_map))
            .sum()
    }
}

fn build_page_rule_mapping(page_rules: &[(usize, usize)]) -> AHashMap<usize, AHashSet<usize>> {
    page_rules
        .iter()
        .fold(AHashMap::new(), |mut acc, (before, after)| {
            acc.entry(*after).or_default().insert(*before);
            acc.entry(*before).or_default();
            acc
        })
}

fn is_valid_page_update(page_update: &[usize], rules: &AHashMap<usize, AHashSet<usize>>) -> bool {
    let mut page_set: AHashSet<usize> = page_update.iter().copied().collect();

    page_update.iter().all(|page| {
        page_set.remove(page);
        rules
            .get(page)
            .into_iter()
            .all(|downstream_pages| downstream_pages.intersection(&page_set).count() == 0)
    })
}

fn find_center_of_page_update(
    page_update: &[usize],
    rules: &AHashMap<usize, AHashSet<usize>>,
) -> usize {
    let page_set: AHashSet<usize> = page_update.iter().copied().collect();
    let target = page_set.len() / 2;

    *page_set
        .iter()
        .find(|page| {
            rules
                .get(page)
                .into_iter()
                .all(|downstream_pages| downstream_pages.intersection(&page_set).count() == target)
        })
        .expect("Exists")
}
