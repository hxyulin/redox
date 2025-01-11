use redox_ast::{
    Attributes, Block, Expr, ExprKind, FunctionDef, Literal, TopLevel, TopLevelKind, Type,
};
use redox_lexer::{Lexer, LexerError, LexerTrait, Span, Token};
use redox_parser_proc_helper::*;
use std::str::FromStr;
use tracing::instrument;

pub struct Parser<'ctx> {
    helper: ParserHelper<'ctx>,
}

impl<'ctx> Parser<'ctx> {
    #[instrument(skip(lexer))]
    pub fn new(lexer: Lexer<'ctx>) -> Self {
        Self {
            helper: ParserHelper::new(lexer),
        }
    }

    #[instrument(skip(source))]
    pub fn with_source(source: &'ctx str) -> Self {
        Self::new(Token::lexer(source))
    }

    #[instrument(skip(self))]
    pub fn parse(&mut self) -> Result<Vec<TopLevel>, ParseError> {
        tracing::trace!("Started parsing");
        let mut top_levels = Vec::new();

        while let Some(tok) = self.helper.advance()? {
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
        self.helper.advance()?;
        let name = match self.helper.current()? {
            Token::Ident(ident) => ident,
            tok => return Err(ParseError::UnexpectedToken(tok)),
        };
        self.helper.advance_no_eof()?;
        self.helper.expect(Token::LeftParen)?;
        self.helper.advance_no_eof()?;
        let arguments = self.parse_typed_argument_list()?;
        tracing::trace!(?arguments, "Parsed arguments");
        self.helper.expect(Token::RightParen)?;

        let return_ty = if let Token::Arrow = self.helper.advance_no_eof()? {
            self.helper.advance_no_eof()?;
            Some(self.parse_type()?)
        } else {
            None
        };
        self.helper.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        // Parse block already consumes the right brace, and we dont' need to check for it here

        Ok(Expr::new(
            ExprKind::FunctionDef(FunctionDef {
                name,
                arguments,
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

        while let Some(tok) = self.helper.advance()? {
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
        redox_parser_proc::parse_rule! {
            expr:expr ";" => Ok(expr),
        }
    }

    #[instrument(skip(self))]
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        tracing::trace!("Parsing expression");
        redox_parser_proc::parse_rule! {
            number:number => Ok(Expr::new(
                ExprKind::Literal(Literal::Number(number)),
                std::ops::Range::default(),
            )),
            ident:ident => Ok(Expr::new(
                ExprKind::Variable(ident),
                std::ops::Range::default(),
            )),
            "return" expr:expr => Ok(Expr::new(
                ExprKind::Return(Some(Box::new(expr))),
                std::ops::Range::default(),
            )),
        }
    }

    /// Parses a type, assuming the first token is consumed
    #[instrument(skip(self))]
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        redox_parser_proc::parse_rule! {
            ident:ident => Ok(Type::from_str(&ident).map_err(|_err| ParseError::UnexpectedToken(Token::Ident(ident)))?),
            "(" ")" => Ok(Type::empty()),
        }
        /*
        match self.helper.current()? {
            Token::LeftParen => match self.helper.advance()?.ok_or(ParseError::UnexpectedEOF)? {
                Token::RightParen => {
                    self.helper.advance()?;
                    Ok(Type::empty())
                }
                _ => unimplemented!("Proper type parsing is not yet implemented!"),
            },
            Token::Ident(ty) => {
                self.helper.advance()?;
                // TODO: Support custom error messages
                Ok(Type::from_str(&ty)
                    .map_err(|_err| ParseError::UnexpectedToken(Token::Ident(ty)))?)
            }
            tok => Err(ParseError::UnexpectedToken(tok)),
        }
        */
    }

    #[instrument(skip(self))]
    fn parse_typed_argument_list(&mut self) -> Result<Vec<(String, Type)>, ParseError> {
        tracing::trace!("Parsing typed argument list");

        // For the first token, we dont error if it is not an ident
        match self.helper.current() {
            Ok(Token::Ident(_)) => (),
            Ok(_) | Err(ParseError::UnexpectedEOF) => return Ok(Vec::new()),
            Err(err) => return Err(err),
        }

        let mut args = Vec::new();
        loop {
            match self.helper.current()? {
                Token::Ident(name) => {
                    self.helper.advance()?;
                    self.helper.expect(Token::Colon)?;
                    self.helper.advance()?;
                    let ty = self.parse_type()?;
                    tracing::trace!(?name, ?ty, "Parsed argument");
                    args.push((name, ty));
                }
                tok => return Err(ParseError::UnexpectedToken(tok)),
            }
            match self.helper.current() {
                Ok(Token::Comma) => self.helper.advance()?,
                Ok(_) | Err(ParseError::UnexpectedEOF) => break,
                Err(err) => return Err(err),
            };
        }
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use redox_ast::NumberLiteral;
    use rstest::rstest;

    // Helper function to create test cases
    fn setup_parser(input: &str) -> Parser {
        Parser::with_source(input)
    }

    #[rstest]
    #[case("fn foo() {}", "foo", vec![], None, Block::empty())]
    #[case("fn add(x: i32) {}", "add", vec![("x".to_string(), Type::from_str("i32").unwrap())], None, Block::empty())]
    #[case("fn ret() -> i32 {}", "ret", vec![], Some(Type::from_str("i32").unwrap()), Block::empty())]
    fn test_function_def(
        #[case] input: &str,
        #[case] expected_name: &str,
        #[case] expected_args: Vec<(String, Type)>,
        #[case] expected_return: Option<Type>,
        #[case] expected_body: Block,
    ) {
        let mut parser = setup_parser(input);
        let top_levels = parser.parse().unwrap();
        assert_eq!(top_levels.len(), 1);
        match &top_levels[0].kind {
            TopLevelKind::Expr(expr) => match &expr.kind {
                ExprKind::FunctionDef(def) => {
                    assert_eq!(def.name, expected_name);
                    assert_eq!(def.arguments, expected_args);
                    assert_eq!(def.return_ty, expected_return);
                    assert_eq!(def.body, expected_body);
                }
                _ => panic!("Expected FunctionDef"),
            },
        }
    }

    #[rstest]
    #[case(
        "return 42;",
        ExprKind::Return(Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Number(NumberLiteral::int32(42))),
            std::ops::Range::default()
        ))))
    )]
    #[case("42;", ExprKind::Literal(Literal::Number(NumberLiteral::int32(42))))]
    #[case("x;", ExprKind::Variable("x".to_string()))]
    fn test_statement(#[case] input: &str, #[case] expected_kind: ExprKind) {
        let mut parser = setup_parser(input);
        parser.helper.advance().unwrap(); // Consume first token
        let result = parser.parse_statement().unwrap();
        assert_eq!(result.kind, expected_kind);
    }

    #[rstest]
    #[case("42", ExprKind::Literal(Literal::Number(NumberLiteral::int32(42))))]
    #[case("x", ExprKind::Variable("x".to_string()))]
    #[case(
        "return 42",
        ExprKind::Return(Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Number(NumberLiteral::int32(42))),
            std::ops::Range::default()
        ))))
    )]
    fn test_expression(#[case] input: &str, #[case] expected_kind: ExprKind) {
        let mut parser = setup_parser(input);
        parser.helper.advance().unwrap(); // Consume first token
        let result = parser.parse_expr().unwrap();
        assert_eq!(result.kind, expected_kind);
    }

    #[rstest]
    #[case("i32", Type::from_str("i32").unwrap())]
    #[case("()", Type::empty())]
    fn test_type(#[case] input: &str, #[case] expected: Type) {
        let mut parser = setup_parser(input);
        parser.helper.advance().unwrap(); // Consume first token
        let result = parser.parse_type().unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("", vec![])]
    #[case("x: i32", vec![("x".to_string(), Type::from_str("i32").unwrap())])]
    #[case("x: i32, y: i32", vec![
        ("x".to_string(), Type::from_str("i32").unwrap()),
        ("y".to_string(), Type::from_str("i32").unwrap())
    ])]
    fn test_typed_argument_list(#[case] input: &str, #[case] expected: Vec<(String, Type)>) {
        let mut parser = setup_parser(input);
        parser.helper.advance().unwrap(); // Consume first token
        let result = parser.parse_typed_argument_list().unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("{}", Block::empty())]
    #[case("{ return 42; }", Block {
        statements: vec![Expr::new(
            ExprKind::Return(Some(Box::new(Expr::new(
                ExprKind::Literal(Literal::Number(NumberLiteral::int32(42))),
                std::ops::Range::default()
            )))),
            std::ops::Range::default()
        )],
        attributes: vec![]
    })]
    fn test_block(#[case] input: &str, #[case] expected: Block) {
        let mut parser = setup_parser(input);
        parser.helper.advance().unwrap(); // Consume first token
        let result = parser.parse_block().unwrap();
        assert_eq!(result, expected);
    }

    // Error case tests
    #[rstest]
    #[case(
        "fn 42() {}",
        ParseError::UnexpectedToken(Token::NumberLit(NumberLiteral::int32(42)))
    )]
    #[case("fn foo(x i32) {}", ParseError::UnexpectedToken(Token::Ident("i32".to_string())))]
    fn test_function_def_errors(#[case] input: &str, #[case] expected_error: ParseError) {
        let mut parser = setup_parser(input);
        let result = parser.parse();
        assert!(matches!(result, Err(ref e) if e == &expected_error));
    }
}
