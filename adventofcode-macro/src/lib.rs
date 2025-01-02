use adventofcode_core::{enum_parse_core, problem_day_core, problem_parse_core};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn problem_day(attr: TokenStream, item: TokenStream) -> TokenStream {
    problem_day_core(attr.into(), item.into()).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn problem_parse(attr: TokenStream, item: TokenStream) -> TokenStream {
    problem_parse_core(attr.into(), item.into()).into()
}

#[proc_macro_error]
#[proc_macro_derive(StringParse, attributes(literal))]
pub fn enum_parse(item: TokenStream) -> TokenStream {
    enum_parse_core(item.into()).into()
}
