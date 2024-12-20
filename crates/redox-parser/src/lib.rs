use redox_ast::{Expr, ExprKind, FunctionDef, Literal, TopLevel, TopLevelKind, Type};
use redox_lexer::{Lexer, LexerError, LexerTrait, Span, Token};

pub struct Parser<'ctx> {
    lexer: Lexer<'ctx>,

    // State
    current_tok: Option<(Token, Span)>,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum ParseError {
    LexerError(LexerError),
    UnexpectedEOF,
    UnexpectedToken(Token),
}

impl From<LexerError> for ParseError {
    fn from(err: LexerError) -> Self {
        Self::LexerError(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LexerError(err) => err.fmt(f),
            Self::UnexpectedEOF => write!(f, "Unexpected EOF"),
            Self::UnexpectedToken(tok) => write!(f, "Unexpected token: {tok:?}"),
        }
    }
}

impl<'ctx> Parser<'ctx> {
    pub fn new(lexer: Lexer<'ctx>) -> Self {
        Self {
            lexer,
            current_tok: None,
        }
    }

    pub fn with_source(source: &'ctx str) -> Self {
        Self::new(Token::lexer(source))
    }

    fn advance(&mut self) -> Result<Option<Token>, ParseError> {
        let tok = if let Some(tok) = self.lexer.next() {
            tok?
        } else {
            self.current_tok = None;
            return Ok(None);
        };

        self.current_tok = Some((tok.clone(), self.lexer.span()));
        Ok(Some(tok))
    }

    fn advance_no_eof(&mut self) -> Result<Token, ParseError> {
        self.advance()?.ok_or(ParseError::UnexpectedEOF)
    }

    fn current(&mut self) -> Result<Token, ParseError> {
        self.current_tok
            .as_ref()
            .ok_or(ParseError::UnexpectedEOF)
            .map(|t| t.0.clone())
    }

    fn expect_advance(&mut self, expected: Token) -> Result<Token, ParseError> {
        let tok = self.advance()?.ok_or(ParseError::UnexpectedEOF)?;
        if tok != expected {
            return Err(ParseError::UnexpectedToken(tok));
        }
        Ok(tok)
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let tok = self.current()?;
        if tok != expected {
            return Err(ParseError::UnexpectedToken(tok));
        }
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Vec<TopLevel>, ParseError> {
        let mut top_levels = Vec::new();

        while let Some(tok) = self.advance()? {
            match tok {
                Token::KwFn => top_levels.push(TopLevel::expr(self.parse_function_def()?)),
                _ => todo!(),
            }
        }
        Ok(top_levels)
    }

    fn parse_function_def(&mut self) -> Result<Expr, ParseError> {
        let name = match self.advance_no_eof()? {
            Token::Ident(ident) => ident,
            _ => todo!(),
        };

        self.expect_advance(Token::LeftParen)?;

        // TODO: parse arguments

        self.advance_no_eof()?; // Simulate parsing the arguments
        self.expect(Token::RightParen)?;

        let return_ty = if let Token::Arrow = self.advance_no_eof()? {
            self.advance_no_eof()?;
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(Token::LeftBrace)?;

        // TODO: Parse body

        self.advance()?.ok_or(ParseError::UnexpectedEOF)?; // Simulate parsing the body
        self.expect(Token::RightBrace)?;

        Ok(Expr::new(
            ExprKind::FunctionDef(FunctionDef { name, return_ty }),
            std::ops::Range::default(),
        ))
    }

    fn parse_block(&mut self) -> Result<Vec<TopLevel>, ParseError> {
        unimplemented!()
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        todo!()
    }

    /// Parses a type, assuming the first token is consumed
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.current()? {
            Token::LeftParen => match self.advance()?.ok_or(ParseError::UnexpectedEOF)? {
                Token::RightParen => {
                    self.advance()?;
                    Ok(Type::empty())
                }
                Token::Ident(_) => unimplemented!("Proper type parsing is not yet implemented!"),
                tok => Err(ParseError::UnexpectedToken(tok)),
            },
            tok => Err(ParseError::UnexpectedToken(tok)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_def() {
        let mut parser = Parser::with_source("fn foo() {}");
        let top_levels = parser.parse().unwrap();
        assert_eq!(top_levels.len(), 1);
        assert_eq!(
            top_levels[0],
            TopLevel::expr(Expr::new(
                ExprKind::FunctionDef(FunctionDef {
                    name: "foo".to_string(),
                    return_ty: None,
                }),
                std::ops::Range::default()
            ))
        );
    }
}
