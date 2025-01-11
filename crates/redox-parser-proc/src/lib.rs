use proc_macro::TokenStream;
use std::str::FromStr;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
enum RuleItem {
    Token(redox_lexer::Token),
    NamedRule {
        name: String,
        capture_as: Option<String>,
    },
    Optional(Box<RuleItem>),
    ZeroOrMore(Box<RuleItem>),
    OneOrMore(Box<RuleItem>),
    Sequence(Vec<RuleItem>),
    CustomExpr(syn::Expr),
}

const CAPTURABLE_TOKS: [&str; 2] = ["ident", "number"];

fn to_tok_ident(tok: &redox_lexer::Token) -> TokenStream2 {
    match tok {
        redox_lexer::Token::KwReturn => quote! { ::redox_lexer::Token::KwReturn },
        redox_lexer::Token::Ident(ident) => {
            quote! { ::redox_lexer::Token::Ident(#ident) }
        }
        redox_lexer::Token::LeftParen => quote! { ::redox_lexer::Token::LeftParen },
        redox_lexer::Token::RightParen => quote! { ::redox_lexer::Token::RightParen },
        redox_lexer::Token::LeftBrace => quote! { ::redox_lexer::Token::LeftBrace },
        redox_lexer::Token::RightBrace => quote! { ::redox_lexer::Token::RightBrace },
        redox_lexer::Token::Semicolon => quote! { ::redox_lexer::Token::Semicolon },
        redox_lexer::Token::Colon => quote! { ::redox_lexer::Token::Colon },
        redox_lexer::Token::Comma => quote! { ::redox_lexer::Token::Comma },
        _ => unimplemented!(),
    }
}

impl RuleItem {
    pub fn to_pattern(&self, tokens: &mut TokenStream2) -> bool {
        match self {
            Self::Token(tok) => {
                tokens.append_all(to_tok_ident(tok));
                true
            }
            Self::NamedRule {
                name,
                capture_as: _,
            } => {
                if CAPTURABLE_TOKS.contains(&name.as_str()) {
                    let tok = match name.as_str() {
                        "ident" => quote! { ::redox_lexer::Token::Ident(_) },
                        "number" => quote! { ::redox_lexer::Token::NumberLit(_) },
                        _ => unimplemented!(),
                    };
                    tokens.append_all(tok);
                    return true;
                }
                false
            }
            Self::Sequence(toks) => {
                if toks.is_empty() {
                    tokens.append_all(quote! { _ });
                    return false;
                }

                toks.first().unwrap().to_pattern(tokens)
            }
            _ => {
                tokens.append_all(quote! { _ });
                false
            }
        }
    }
}

impl quote::ToTokens for RuleItem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Token(tok) => {
                let tok: TokenStream2 = match tok {
                    redox_lexer::Token::KwReturn => quote! { ::redox_lexer::Token::KwReturn },
                    redox_lexer::Token::Ident(ident) => {
                        quote! { ::redox_lexer::Token::Ident(#ident) }
                    }
                    redox_lexer::Token::LeftParen => quote! { ::redox_lexer::Token::LeftParen },
                    redox_lexer::Token::RightParen => quote! { ::redox_lexer::Token::RightParen },
                    redox_lexer::Token::LeftBrace => quote! { ::redox_lexer::Token::LeftBrace },
                    redox_lexer::Token::RightBrace => quote! { ::redox_lexer::Token::RightBrace },
                    redox_lexer::Token::Semicolon => quote! { ::redox_lexer::Token::Semicolon },
                    redox_lexer::Token::Colon => quote! { ::redox_lexer::Token::Colon },
                    redox_lexer::Token::Comma => quote! { ::redox_lexer::Token::Comma },
                    _ => unimplemented!(),
                }
                .into();
                tokens.append_all(quote! {
                    self.helper.expect(#tok)?;
                    self.helper.advance()?;
                });
            }
            Self::NamedRule { name, capture_as } => {
                if let Some(capture) = capture_as {
                    let capture = format_ident!("{}", capture);
                    tokens.append_all(quote! {
                        let #capture =
                    });
                } else {
                    tokens.append_all(quote! {
                        let _ =
                    });
                };

                if CAPTURABLE_TOKS.contains(&name.as_str()) {
                    let tok = match name.as_str() {
                        "ident" => quote! { Ident(ident) => ident },
                        "number" => quote! { NumberLit(number) => number  },
                        _ => unimplemented!(),
                    };
                    tokens.append_all(quote! {
                        match self.helper.current()? {
                            ::redox_lexer::Token::#tok,
                            tok => return Err(::redox_parser_proc_helper::ParseError::UnexpectedToken(tok)),
                        };
                        self.helper.advance()?;
                    });
                    return;
                }

                let fn_name = format_ident!("parse_{}", name);
                println!("{}", fn_name.to_string());
                tokens.append_all(quote! {
                    self.#fn_name()?;
                });
            }
            Self::Sequence(toks) => {
                for tok in toks {
                    tok.to_tokens(tokens);
                }
            }
            Self::CustomExpr(expr) => {
                tokens.append_all(expr.into_token_stream());
            }
            _ => unimplemented!(),
        }
    }
}

impl syn::parse::Parse for RuleItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rules: Vec<RuleItem> = Vec::new();
        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Ident) {
                let ident: syn::Ident = input.parse()?;

                let mut capture_as = Option::<String>::None;
                let lookahead = input.lookahead1();
                if lookahead.peek(syn::Token![:]) {
                    let _: syn::Token![:] = input.parse()?;
                    let capture: syn::Ident = input.parse()?;
                    capture_as = Some(capture.to_string());
                }
                rules.push(RuleItem::NamedRule {
                    name: ident.to_string(),
                    capture_as,
                })
            } else if lookahead.peek(syn::LitStr) {
                let str = input.parse::<syn::LitStr>()?;
                let tok = redox_lexer::Token::from_str(&str.value())
                    .map_err(|_| syn::Error::new(str.span(), "Invalid token"))?;
                rules.push(RuleItem::Token(tok));
            } else if lookahead.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                rules.push(content.parse::<RuleItem>()?);
            } else if lookahead.peek(syn::Token![@]) {
                // We just parse the expression
                let _: syn::Token![@] = input.parse()?;
                let expr = input.parse::<syn::Expr>()?;
                rules.push(RuleItem::CustomExpr(expr));
            } else {
                break if rules.len() == 1 {
                    Ok(rules.pop().unwrap())
                } else {
                    Ok(RuleItem::Sequence(rules))
                };
            }

            // We check for modifiers here
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Token![*]) {
                let _: syn::Token![*] = input.parse()?;
                let last = rules.pop().unwrap();
                rules.push(RuleItem::ZeroOrMore(Box::new(last)));
            } else if lookahead.peek(syn::Token![+]) {
                let _: syn::Token![+] = input.parse()?;
                let last = rules.pop().unwrap();
                rules.push(RuleItem::OneOrMore(Box::new(last)));
            } else if lookahead.peek(syn::Token![?]) {
                let _: syn::Token![?] = input.parse()?;
                let last = rules.pop().unwrap();
                rules.push(RuleItem::Optional(Box::new(last)));
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
struct RuleOutput {
    code: syn::Expr,
}

impl syn::parse::Parse for RuleOutput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code = input.parse()?;
        Ok(RuleOutput { code })
    }
}

impl quote::ToTokens for RuleOutput {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.code.to_tokens(tokens);
    }
}

impl quote::ToTokens for RuleMatcher {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.item.to_tokens(tokens);
        self.output.to_tokens(tokens);
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
struct RuleMatcher {
    item: RuleItem,
    output: RuleOutput,
}

impl syn::parse::Parse for RuleMatcher {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item = input.parse()?;
        let _: syn::Token![=>] = input.parse()?;
        let output = input.parse()?;
        Ok(RuleMatcher { item, output })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Rule {
    name: String,
    matchers: Vec<RuleMatcher>,
}

impl syn::parse::Parse for Rule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = "TODO".to_string();
        let matchers = input
            .parse_terminated(RuleMatcher::parse, syn::Token![,])?
            .into_iter()
            .collect();
        Ok(Rule { name, matchers })
    }
}

impl quote::ToTokens for Rule {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let _name = &self.name;
        let matchers = &self.matchers;
        if matchers.len() == 1 {
            let matcher = &matchers[0];
            tokens.append_all(matcher.to_token_stream());
        } else {
            let mut pattern_matchers = matchers
                .iter()
                .map(|matcher| {
                    let mut pattern = TokenStream2::default();
                    let is_specific = matcher.item.to_pattern(&mut pattern);
                    (pattern, is_specific, matcher)
                })
                .collect::<Vec<_>>();
            pattern_matchers.sort_by_key(|(_, is_specific, _)| !is_specific);

            // if there is more than one that is not specific, then we error because the grammer is ambiguous
            let non_specific = pattern_matchers
                .iter()
                .filter(|(_, is_specific, _)| !is_specific)
                .count();
            if non_specific > 1 {
                panic!("Grammar is ambiguous");
            }

            let patterns = pattern_matchers
                .iter()
                .map(|(pattern, _, _)| pattern)
                .collect::<Vec<_>>();

            let matchers = pattern_matchers
                .iter()
                .map(|(_, _, matcher)| matcher)
                .collect::<Vec<_>>();

            tokens.append_all(quote! {
                match self.helper.current()? {
                    #(#patterns => {#matchers},)*
                    tok => return Err(::redox_parser_proc_helper::ParseError::UnexpectedToken(tok)),
                }
            });
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Parser {
    rules: Vec<Rule>,
}

impl Parser {
    fn get_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|rule| rule.name == name)
    }
}

#[proc_macro]
pub fn parse_rule(input: TokenStream) -> TokenStream {
    let rule = syn::parse_macro_input!(input as Rule);
    println!("{}", (quote! { #rule }));
    rule.to_token_stream().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    // Frontend tests (parsing)
    #[rstest]
    #[case(
        quote! { "return" },
        RuleItem::Token(redox_lexer::Token::KwReturn)
    )]
    #[case(
        quote! { expr },
        RuleItem::NamedRule { 
            name: "expr".to_string(), 
            capture_as: None 
        }
    )]
    #[case(
        quote! { expr:captured },
        RuleItem::NamedRule { 
            name: "expr".to_string(), 
            capture_as: Some("captured".to_string()) 
        }
    )]
    #[case(
        quote! { ("," expr)* },
        RuleItem::ZeroOrMore(Box::new(RuleItem::Sequence(vec![
            RuleItem::Token(redox_lexer::Token::Comma),
            RuleItem::NamedRule { 
                name: "expr".to_string(), 
                capture_as: None 
            }
        ])))
    )]
    fn test_rule_item_parsing(#[case] input: TokenStream2, #[case] expected: RuleItem) {
        let parsed = syn::parse2::<RuleItem>(input).unwrap();
        assert_eq!(parsed, expected);
    }

    // Test rule matcher parsing
    #[rstest]
    #[case(
        quote! { "return" expr => Ok(expr) },
        RuleMatcher {
            item: RuleItem::Sequence(vec![
                RuleItem::Token(redox_lexer::Token::KwReturn),
                RuleItem::NamedRule { 
                    name: "expr".to_string(), 
                    capture_as: None 
                }
            ]),
            output: RuleOutput {
                code: syn::parse2::<syn::Expr>(quote! { Ok(expr) }).unwrap()
            }
        }
    )]
    fn test_rule_matcher_parsing(#[case] input: TokenStream2, #[case] expected: RuleMatcher) {
        let parsed = syn::parse2::<RuleMatcher>(input).unwrap();
        assert_eq!(parsed, expected);
    }

    // Test pattern generation
    #[rstest]
    #[case(
        RuleItem::Token(redox_lexer::Token::KwReturn),
        quote! { ::redox_lexer::Token::KwReturn },
        true
    )]
    #[case(
        RuleItem::NamedRule { 
            name: "ident".to_string(), 
            capture_as: None 
        },
        quote! { ::redox_lexer::Token::Ident(_) },
        true
    )]
    fn test_pattern_generation(
        #[case] rule: RuleItem, 
        #[case] expected_pattern: TokenStream2,
        #[case] expected_specific: bool
    ) {
        let mut pattern = TokenStream2::new();
        let is_specific = rule.to_pattern(&mut pattern);
        assert_eq!(pattern.to_string(), expected_pattern.to_string());
        assert_eq!(is_specific, expected_specific);
    }

    // Test code generation
    #[rstest]
    #[case(
        RuleItem::Token(redox_lexer::Token::KwReturn),
        quote! {
            self.helper.expect(::redox_lexer::Token::KwReturn)?;
            self.helper.advance()?;
        }
    )]
    #[case(
        RuleItem::NamedRule { 
            name: "ident".to_string(), 
            capture_as: Some("var".to_string()) 
        },
        quote! {
            let var = match self.helper.current()? {
                ::redox_lexer::Token::Ident(ident) => ident,
                tok => return Err(::redox_parser_proc_helper::ParseError::UnexpectedToken(tok)),
            };
            self.helper.advance()?;
        }
    )]
    fn test_code_generation(#[case] rule: RuleItem, #[case] expected_code: TokenStream2) {
        let generated = quote! { #rule };
        println!("{}", generated.to_string());
        assert_eq!(generated.to_string(), expected_code.to_string());
    }

    // Test error cases
    #[rstest]
    #[case(quote! { "invalid_token" })]
    #[case(quote! { expr:123 })] // Invalid capture name
    #[should_panic]
    fn test_parsing_errors(#[case] input: TokenStream2) {
        let _parsed = syn::parse2::<RuleItem>(input).unwrap();
    }

    // Test Rule ambiguity detection
    #[rstest]
    #[case(
        quote! {
            expr => Ok(expr),
            expr => Ok(expr)
        }
    )]
    #[should_panic(expected = "Grammar is ambiguous")]
    fn test_ambiguous_grammar(#[case] input: TokenStream2) {
        let rule = syn::parse2::<Rule>(input).unwrap();
        let _generated = quote! { #rule };
    }

    // Test complete rule parsing and generation
    #[rstest]
    #[case(
        quote! {
            "return" expr:expr ";" => Ok(Expr::Return(expr))
        },
        None // Add expected output when you want to test specific generation
    )]
    fn test_complete_rule(#[case] input: TokenStream2, #[case] expected: Option<TokenStream2>) {
        let rule = syn::parse2::<Rule>(input).unwrap();
        let generated = rule.to_token_stream();
        if let Some(expected) = expected {
            assert_eq!(generated.to_string(), expected.to_string());
        };
    }

    // Test repetition handling
    #[rstest]
    #[case(
        quote! { ("," expr)* },
        RuleItem::ZeroOrMore(Box::new(RuleItem::Sequence(vec![
            RuleItem::Token(redox_lexer::Token::Comma),
            RuleItem::NamedRule { 
                name: "expr".to_string(), 
                capture_as: None 
            }
        ])))
    )]
    #[case(
        quote! { expr+ },
        RuleItem::OneOrMore(Box::new(RuleItem::NamedRule { 
            name: "expr".to_string(), 
            capture_as: None 
        }))
    )]
    fn test_repetition_parsing(#[case] input: TokenStream2, #[case] expected: RuleItem) {
        let parsed = syn::parse2::<RuleItem>(input).unwrap();
        assert_eq!(parsed, expected);
    }
}
