use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, ItemFn, ReturnType, Stmt, Type, parse2};

pub fn problem_day_core(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        abort!(args, "Must specify exactly zero arguments.")
    }

    let mut run = match parse2::<ItemFn>(input) {
        Ok(run) => run,
        Err(e) => return e.to_compile_error(),
    };

    let output_type = match run.sig.output.clone() {
        ReturnType::Default => {
            abort!(
                run.to_token_stream(),
                "Must have a return type that implements Into<ProblemResult>"
            )
        }
        ReturnType::Type(_, t) => t,
    };

    let name = &run.sig.ident.to_string();
    if name != "run" {
        abort!(run.sig.ident, "Name must be \"run\"")
    }

    if run.sig.inputs.len() != 2 {
        abort!(run.sig.inputs, "Exactly two arguments required.")
    }

    let (input_type, input_name) = match run.sig.inputs.get(0).expect("Already verified length") {
        FnArg::Receiver(receiver) => {
            abort!(receiver.to_token_stream(), "Should be it's own input")
        }
        FnArg::Typed(pat_type) => (pat_type.ty.clone(), pat_type.pat.clone()),
    };

    let command_line_arguments = match run.sig.inputs.get(1).expect("Already verified length") {
        FnArg::Receiver(receiver) => {
            abort!(receiver.to_token_stream(), "Should be it's own input")
        }
        FnArg::Typed(pat_type) => match *pat_type.ty.clone() {
            syn::Type::Reference(type_reference) => type_reference.elem,
            _ => abort!(
                pat_type.ty.to_token_stream(),
                "Requires borrowed CommandLineLine arg",
            ),
        },
    };

    run.sig.output = parse2::<ReturnType>(quote! { -> Self::Output }).expect("Works");
    run.sig.inputs[0] = parse2::<FnArg>(quote! { self }).expect("Works");
    run.block.stmts.insert(
        0,
        parse2::<Stmt>(quote! { let #input_name = self; }).expect("Works"),
    );

    quote! {
        impl Problem<#command_line_arguments> for #input_type {
            type Output = #output_type;

            #run
        }
    }
}

#[test]
fn implements_problem() {
    let before = quote! {
        fn run(input: Day26, arguments: &CommandLineArguments) -> usize {
            0
        }
    };
    let after = problem_day_core(quote!(), before);
    assert_eq!(
        after.to_string(),
        "impl Problem < CommandLineArguments > for Day26 { type Output = usize ; fn run (self , arguments : & CommandLineArguments) -> Self :: Output { let input = self ; 0 } }"
    );
}

pub fn problem_parse_core(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        abort!(args, "Must specify exactly zero arguments.")
    }

    let mut run = match parse2::<ItemFn>(input) {
        Ok(run) => run,
        Err(e) => return e.to_compile_error(),
    };

    let target = match &mut run.sig.output {
        ReturnType::Type(_, t) => match t.as_mut() {
            syn::Type::ImplTrait(type_impl_trait) => {
                if type_impl_trait.bounds.len() != 1 {
                    abort!(
                        type_impl_trait.bounds.to_token_stream(),
                        "Expected exactly 1 trait bound of Parser"
                    )
                }

                match type_impl_trait.bounds.first_mut().expect("Bounds checked") {
                    syn::TypeParamBound::Trait(trait_bound) => {
                        if trait_bound.path.segments.len() != 1 {
                            abort!(
                                trait_bound.path.segments.to_token_stream(),
                                "Expected 1 trait bound of Parser"
                            )
                        }

                        match &mut trait_bound
                            .path
                            .segments
                            .first_mut()
                            .expect("Bounds checked")
                            .arguments
                        {
                            syn::PathArguments::AngleBracketed(
                                angle_bracketed_generic_arguments,
                            ) => {
                                if angle_bracketed_generic_arguments.args.len() != 4 {
                                    abort!(
                                        angle_bracketed_generic_arguments.args.to_token_stream(),
                                        "Expected 4 type parameters for the parser"
                                    )
                                }

                                match angle_bracketed_generic_arguments
                                    .args
                                    .get_mut(2)
                                    .expect("Bounds Checked")
                                {
                                    syn::GenericArgument::Type(target) => target,
                                    _ => abort!(
                                        angle_bracketed_generic_arguments.args.to_token_stream(),
                                        "Unexpected argument, expected type but found something else"
                                    ),
                                }
                            }
                            _ => abort!(
                                trait_bound.path.segments.to_token_stream(),
                                "Expected angle bracket path."
                            ),
                        }
                    }
                    _ => abort!(
                        type_impl_trait.bounds.to_token_stream(),
                        "Expected a type parameter"
                    ),
                }
            }
            _ => abort!(t.to_token_stream(), "Expected an impl Parser"),
        },
        ReturnType::Default => abort!(
            run.sig.output.to_token_stream(),
            "Expected an impl Parser return, not unit"
        ),
    };

    let day = target.clone();
    *target = parse2::<Type>(quote! { Self }).expect("Works");

    quote! {
        impl StringParse for #day {
            #run
        }
    }
}

#[test]
fn implements_string_parse() {
    let before = quote! {
        fn parse<'a>() -> impl Parser<'a, &'a str, Day26, extra::Err<Rich<'a, char>>> {
            just("").to(Day26)
        }
    };
    let after = problem_parse_core(quote!(), before);
    assert_eq!(
        after.to_string(),
        "impl StringParse for Day26 { fn parse < 'a > () -> impl Parser < 'a , & 'a str , Self , extra :: Err < Rich < 'a , char > > > { just (\"\") . to (Day26) } }"
    );
}
