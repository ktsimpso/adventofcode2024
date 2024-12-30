use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, ItemFn, ItemStruct, ReturnType, parse_quote, parse_str, parse2};

pub fn problem_day_core(args: TokenStream, input: TokenStream) -> TokenStream {
    let arguments = args
        .clone()
        .into_iter()
        .map(|argument| argument.to_string())
        .collect::<Vec<_>>();

    if arguments.len() != 1 {
        abort!(args, "Must specify exactly one argument.")
    }

    let day = &arguments[0];

    match day.chars().next() {
        Some(c) => {
            if !c.is_uppercase() {
                abort!(args, "Day must be capitalized")
            }
        }
        None => abort!(args, "Argument must be at least one character."),
    }

    let day = match parse_str::<Ident>(day) {
        Ok(day) => day,
        Err(e) => return e.to_compile_error(),
    };

    let day_struct: ItemStruct = parse_quote! {
        pub struct #day {}
    };

    let mut run = match parse2::<ItemFn>(input) {
        Ok(run) => run,
        Err(e) => return e.to_compile_error(),
    };

    let output_type = match run.sig.output.clone() {
        syn::ReturnType::Default => {
            abort!(
                run.to_token_stream(),
                "Must have a return type that implements Into<ProblemResult>"
            )
        }
        syn::ReturnType::Type(_, t) => t,
    };

    let name = &run.sig.ident.to_string();
    if name != "run" {
        abort!(run.sig.ident, "Name must be \"run\"")
    }

    if run.sig.inputs.len() != 2 {
        abort!(run.sig.inputs, "Exactly two arguments required.")
    }

    let input = match run.sig.inputs.get(0).expect("Already verified length") {
        syn::FnArg::Receiver(receiver) => {
            abort!(receiver.to_token_stream(), "Should be it's own input")
        }
        syn::FnArg::Typed(pat_type) => &pat_type.ty,
    };

    let command_line_arguments = match run.sig.inputs.get(1).expect("Already verified length") {
        syn::FnArg::Receiver(receiver) => {
            abort!(receiver.to_token_stream(), "Should be it's own input")
        }
        syn::FnArg::Typed(pat_type) => match *pat_type.ty.clone() {
            syn::Type::Reference(type_reference) => type_reference.elem,
            _ => abort!(
                pat_type.ty.to_token_stream(),
                "Requires borrowed CommandLineLine arg",
            ),
        },
    };

    run.sig.output = parse2::<ReturnType>(quote! { -> Self::Output }).expect("Works");

    quote! {
        #day_struct

        impl Problem<#input, #command_line_arguments> for #day {
            type Output = #output_type;

            #run
        }
    }
}

#[test]
fn adds_struct_and_impl() {
    let before = quote! {
        fn run(input: Input, arguments: &CommandLineArguments) -> usize {
            0
        }
    };
    let after = problem_day_core(quote!(Day26), before);
    assert_eq!(
        after.to_string(),
        "pub struct Day26 { } impl Problem < Input , CommandLineArguments > for Day26 { type Output = usize ; fn run (input : Input , arguments : & CommandLineArguments) -> Self :: Output { 0 } }"
    );
}
