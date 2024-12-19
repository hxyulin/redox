use std::ops::Range;
pub use logos::{Logos as LexerTrait, Span};
use logos::Logos;

#[derive(Default, Debug, Clone, PartialEq, thiserror::Error)]
pub enum LexerError {
    #[default]
    NonAsciiCharacter,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonAsciiCharacter => write!(f, "Non-ascii character"),
        }
    }
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = LexerError)]
#[logos(skip r"[\s\n\r]+")]
pub enum Token {
    #[token("fn")]
    KwFn,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[token("->")]
    Arrow,
}

#[cfg(test)]
mod tests {
    use {
        super::{LexerTrait, Token},
        pretty_assertions::assert_eq,
        rstest::rstest,
    };

    #[rstest]
    #[case("fn", Token::KwFn)]
    #[case("(", Token::LeftParen)]
    #[case(")", Token::RightParen)]
    #[case("{", Token::LeftBrace)]
    #[case("}", Token::RightBrace)]
    #[case("foo", Token::Ident("foo".to_string()))]
    #[case("->", Token::Arrow)]
    fn test_lexing_tok(#[case] input: &str, #[case] expected: Token) {
        let mut lexer = Token::lexer(input);
        let tok = lexer.next();
        assert_eq!(tok, Some(Ok(expected)));
    }

    #[rstest]
    #[case("fn main() {}", vec![
        Token::KwFn, 
        Token::Ident("main".to_string()), 
        Token::LeftParen, 
        Token::RightParen, 
        Token::LeftBrace, 
        Token::RightBrace
    ])]
    fn test_lexing_seq(#[case] input: &str, #[case] expected: Vec<Token>) {
        let mut lexer = Token::lexer(input);
        let mut tokens = Vec::new();
        while let Some(tok) = lexer.next() {
            tokens.push(tok.unwrap());
        }
        assert_eq!(tokens, expected);
    }
}

pub type Lexer<'source> = logos::Lexer<'source, Token>;
