use redox_ast::{
    Attributes, Block, Expr, ExprKind, FunctionDef, Literal, TopLevel, TopLevelKind, Type,
};
use redox_lexer::{Lexer, LexerError, LexerTrait, Span, Token};
use std::str::FromStr;
use tracing::instrument;

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
    #[instrument(skip(lexer))]
    pub fn new(lexer: Lexer<'ctx>) -> Self {
        Self {
            lexer,
            current_tok: None,
        }
    }

    #[instrument(skip(source))]
    pub fn with_source(source: &'ctx str) -> Self {
        Self::new(Token::lexer(source))
    }

    #[instrument(skip(self))]
    fn advance(&mut self) -> Result<Option<Token>, ParseError> {
        tracing::trace!("Advance");
        let tok = if let Some(tok) = self.lexer.next() {
            tok?
        } else {
            self.current_tok = None;
            return Ok(None);
        };

        self.current_tok = Some((tok.clone(), self.lexer.span()));
        Ok(Some(tok))
    }

    #[instrument(skip(self))]
    fn advance_no_eof(&mut self) -> Result<Token, ParseError> {
        tracing::trace!("Advance ensuring no EOF");
        self.advance()?.ok_or(ParseError::UnexpectedEOF)
    }

    #[instrument(skip(self))]
    fn current(&mut self) -> Result<Token, ParseError> {
        tracing::trace!("Current token");
        self.current_tok
            .as_ref()
            .ok_or(ParseError::UnexpectedEOF)
            .map(|t| t.0.clone())
    }

    #[instrument(skip(self))]
    fn expect_advance(&mut self, expected: Token) -> Result<Token, ParseError> {
        tracing::trace!(?expected, "Expecting advance");
        let tok = self.advance()?.ok_or(ParseError::UnexpectedEOF)?;
        if tok != expected {
            return Err(ParseError::UnexpectedToken(tok));
        }
        Ok(tok)
    }

    #[instrument(skip(self))]
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        tracing::trace!(?expected, "Expecting token");
        let tok = self.current()?;
        if tok != expected {
            return Err(ParseError::UnexpectedToken(tok));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn parse(&mut self) -> Result<Vec<TopLevel>, ParseError> {
        tracing::trace!("Started parsing");
        let mut top_levels = Vec::new();

        while let Some(tok) = self.advance()? {
            match tok {
                Token::KwFn => top_levels.push(TopLevel::expr(self.parse_function_def()?)),
                _ => todo!(),
            }
        }
        Ok(top_levels)
    }

    #[instrument(skip(self))]
    fn parse_function_def(&mut self) -> Result<Expr, ParseError> {
        tracing::trace!("Parsing function definition");
        let attributes: Attributes = Vec::new();
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
        let body = self.parse_block()?;
        // Parse block already consumes the right brace, and we dont' need to check for it here

        Ok(Expr::new(
            ExprKind::FunctionDef(FunctionDef {
                name,
                return_ty,
                attributes,
                body,
            }),
            std::ops::Range::default(),
        ))
    }

    /// Parses a block, assuming the current token is the left brace
    #[instrument(skip(self))]
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        tracing::trace!("Parsing block");
        let mut statements = Vec::new();

        while let Some(tok) = self.advance()? {
            match tok {
                Token::RightBrace => break,
                Token::LeftBrace => unimplemented!("Nested blocks are not yet supported"),
                _ => {
                    let statement = self.parse_statement()?;
                    statements.push(statement);
                }
            }
        }

        Ok(Block {
            statements,
            attributes: Vec::new(),
        })
    }

    #[instrument(skip(self))]
    fn parse_statement(&mut self) -> Result<Expr, ParseError> {
        tracing::trace!("Parsing statement");
        // TODO: We need to respect semiclons
        let res = match self.current()? {
            Token::KwReturn => {
                self.advance()?;
                let expr = self.parse_expr()?;
                Expr::new(
                    ExprKind::Return(Some(Box::new(expr))),
                    std::ops::Range::default(),
                )
            }
            _ => self.parse_expr()?,
        };

        self.expect(Token::Semicolon)?;
        // Consume the semicolon
        self.advance()?;
        Ok(res)
    }

    #[instrument(skip(self))]
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        tracing::trace!("Parsing expression");
        match self.current()? {
            Token::NumberLit(num) => {
                self.advance()?;
                Ok(Expr::new(
                    ExprKind::Literal(Literal::Number(num)),
                    std::ops::Range::default(),
                ))
            }
            Token::KwReturn => self.parse_statement(),
            tok => todo!("Unexpected token (unimplemented): {:?}", tok),
        }
    }

    /// Parses a type, assuming the first token is consumed
    #[instrument(skip(self))]
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        tracing::trace!("Parsing type");
        match self.current()? {
            Token::LeftParen => match self.advance()?.ok_or(ParseError::UnexpectedEOF)? {
                Token::RightParen => {
                    self.advance()?;
                    Ok(Type::empty())
                }
                _ => unimplemented!("Proper type parsing is not yet implemented!"),
            },
            Token::Ident(ty) => {
                self.advance()?;
                // TODO: Support custom error messages
                Ok(Type::from_str(&ty)
                    .map_err(|_err| ParseError::UnexpectedToken(Token::Ident(ty)))?)
            }
            tok => Err(ParseError::UnexpectedToken(tok)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redox_ast::Block;

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
                    attributes: Vec::new(),
                    body: Block::empty(),
                }),
                std::ops::Range::default()
            ))
        );
    }
}
