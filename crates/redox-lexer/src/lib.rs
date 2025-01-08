use std::ops::Range;
pub use logos::{Logos as LexerTrait, Span};
use logos::Logos;

#[derive(Default, Debug, Clone, PartialEq, thiserror::Error)]
pub enum LexerError {
    #[default]
    NonAsciiCharacter,
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonAsciiCharacter => write!(f, "Non-ascii character"),
            Self::ParseIntError(err) => write!(f, "Parse int error: {}", err),
            Self::ParseFloatError(err) => write!(f, "Parse float error: {}", err),
        }
    }
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = LexerError)]
#[logos(skip r"[\s\n\r]+")]
pub enum Token {
    #[token("fn")]
    KwFn,
    #[token("return")]
    KwReturn,

    #[token(";")]
    Semicolon,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("->")]
    Arrow,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[0-9]+", parse_num_literal)]
    NumberLit(redox_ast::NumberLiteral),
}

fn parse_num_literal(lex: &mut Lexer) -> Result<redox_ast::NumberLiteral, LexerError> {
    let mut num = lex.slice().to_string();
    let mut radix = 10;
    if num.starts_with("0x") {
        num = num[2..].to_string();
        radix = 16;
    }
    let num = u64::from_str_radix(&num, radix).map_err(|err| LexerError::ParseIntError(err))?;
    Ok(redox_ast::NumberLiteral::int32(num))
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
