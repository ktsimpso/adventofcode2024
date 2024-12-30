use adventofcode_core::problem_day_core;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn problem_day(attr: TokenStream, item: TokenStream) -> TokenStream {
    problem_day_core(attr.into(), item.into()).into()
}
