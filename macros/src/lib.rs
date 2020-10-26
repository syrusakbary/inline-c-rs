use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
pub fn c(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let input_as_string = reconstruct(input);

    quote!(
        {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let result = inline_c::run_c(
                #input_as_string,
                &mut stdout,
                &mut stderr
            );

            (result, stdout, stderr)
        }
    )
    .into()
}

fn reconstruct(input: TokenStream) -> String {
    use proc_macro2::{Delimiter, Spacing, TokenTree::*};

    let mut output = String::new();
    let mut iterator = input.into_iter().peekable();

    loop {
        match iterator.next() {
            Some(Punct(token)) => {
                let token_value = token.as_char();
                output.push(token_value);

                match token_value {
                    '#' => match iterator.peek() {
                        Some(Ident(include)) if include.to_string() == "include".to_string() => {
                            iterator.next();

                            let opening;
                            let closing;

                            match iterator.next() {
                                Some(Punct(punct)) => {
                                    opening = punct.as_char();

                                    if opening == '"' {
                                        closing = '"';
                                    } else {
                                        closing = '>';
                                    }
                                }

                                Some(token) => panic!(
                                    "Invalid opening token after `#include`, received `{:?}`.",
                                    token
                                ),

                                None => panic!("`#include` must be followed by `<` or `\"`."),
                            }

                            output.push_str("include");
                            output.push(' ');
                            output.push(opening);

                            loop {
                                match iterator.next() {
                                    Some(Punct(punct)) => {
                                        let punct = punct.as_char();

                                        if punct == closing {
                                            break;
                                        }

                                        output.push(punct)
                                    }
                                    Some(Ident(ident)) => output.push_str(&ident.to_string()),
                                    token => panic!(
                                        "Invalid token in `#include` value, with `{:?}`.",
                                        token
                                    ),
                                }
                            }

                            output.push(closing);
                            output.push('\n');
                        }
                        _ => {}
                    },

                    ';' => {
                        output.push('\n');
                    }

                    _ => {
                        if token.spacing() == Spacing::Alone {
                            output.push(' ');
                        }
                    }
                }
            }

            Some(Ident(ident)) => {
                output.push_str(&ident.to_string());
                output.push(' ');
            }

            Some(Group(group)) => {
                let group_output = reconstruct(group.stream());

                match group.delimiter() {
                    Delimiter::Parenthesis => {
                        output.push('(');
                        output.push_str(&group_output);
                        output.push(')');
                    }

                    Delimiter::Brace => {
                        output.push('{');
                        output.push('\n');
                        output.push_str(&group_output);
                        output.push('\n');
                        output.push('}');
                    }

                    Delimiter::Bracket => {
                        output.push('[');
                        output.push_str(&group_output);
                        output.push(']');
                    }

                    Delimiter::None => {
                        output.push_str(&group_output);
                    }
                }
            }

            Some(token) => {
                output.push_str(&token.to_string());
            }

            None => break,
        }
    }

    output
}
