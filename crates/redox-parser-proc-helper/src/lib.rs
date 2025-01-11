use redox_lexer::{Lexer, LexerError, Span, Token};
use tracing::instrument;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ParseError {
    LexerError(LexerError),
    UnexpectedEOF,
    UnexpectedToken(Token),
    UnclosedComment,
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
            Self::UnclosedComment => write!(f, "Unclosed comment"),
        }
    }
}

pub struct ParserHelper<'ctx> {
    lexer: Lexer<'ctx>,

    // State
    current_tok: Option<(Token, Span)>,
}

impl<'ctx> ParserHelper<'ctx> {
    pub fn new(lexer: Lexer<'ctx>) -> Self {
        Self {
            lexer,
            current_tok: None,
        }
    }

    #[instrument(skip(self))]
    pub fn advance(&mut self) -> Result<Option<Token>, ParseError> {
        tracing::trace!("Advance");
        let tok = if let Some(tok) = self.lexer.next() {
            let mut tok = tok?;
            if tok == Token::OpenComment {
                tracing::trace!("Open comment");
                let mut broken = false;
                while let Some(tok) = self.lexer.next() {
                    tracing::trace!(?tok, "Comment token");
                    if tok? == Token::CloseComment {
                        tracing::trace!("Close comment");
                        broken = true;
                        break;
                    }
                }

                if !broken {
                    return Err(ParseError::UnclosedComment);
                }

                if let Some(new_tok) = self.lexer.next() {
                    tok = new_tok?;
                } else {
                    self.current_tok = None;
                    return Ok(None);
                };
            }
            tok
        } else {
            self.current_tok = None;
            return Ok(None);
        };

        self.current_tok = Some((tok.clone(), self.lexer.span()));
        Ok(Some(tok))
    }

    #[instrument(skip(self))]
    pub fn advance_no_eof(&mut self) -> Result<Token, ParseError> {
        tracing::trace!("Advance ensuring no EOF");
        self.advance()?.ok_or(ParseError::UnexpectedEOF)
    }

    #[instrument(skip(self))]
    pub fn current(&mut self) -> Result<Token, ParseError> {
        tracing::trace!("Current token");
        self.current_tok
            .as_ref()
            .ok_or(ParseError::UnexpectedEOF)
            .map(|t| t.0.clone())
    }

    #[instrument(skip(self))]
    pub fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        tracing::trace!(?expected, "Expecting token");
        let tok = self.current()?;
        if tok != expected {
            return Err(ParseError::UnexpectedToken(tok));
        }
        Ok(())
    }
}
