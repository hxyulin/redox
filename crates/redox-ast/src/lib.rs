use std::ops::Range;

pub mod literal;
pub mod types;
pub use {literal::*, types::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute {}

pub type Attributes = Vec<Attribute>;

#[derive(Debug, Clone, PartialEq)]
pub struct Wrapped<T> {
    pub kind: T,
    pub span: Range<usize>,
    pub ty: Option<Type>,
}

impl<T> Wrapped<T> {
    pub fn new(kind: T, span: Range<usize>) -> Self {
        Self {
            kind,
            span,
            ty: None,
        }
    }
}

impl ExprKind {
    fn get_children(&self) -> Vec<Box<Expr>> {
        match self {
            ExprKind::Literal(_) => Vec::new(),
            ExprKind::Return(expr) => {
                if let Some(expr) = expr {
                    vec![expr.clone()]
                } else {
                    Vec::new()
                }
            }
            ExprKind::FunctionDef(function_def) => function_def
                .body
                .statements
                .iter()
                .map(|expr| Box::new(expr.clone()))
                .collect(),
        }
    }
    fn is_top_level(&self) -> bool {
        matches!(self, ExprKind::FunctionDef { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Return(Option<Box<Expr>>),
    FunctionDef(FunctionDef),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelKind {
    Expr(Box<Expr>),
}

pub type Expr = Wrapped<ExprKind>;
pub type TopLevel = Wrapped<TopLevelKind>;
pub type Ast = Vec<TopLevel>;

impl TopLevel {
    pub fn expr(expr: Expr) -> Self {
        let range = expr.span.clone();
        Self::new(TopLevelKind::Expr(Box::new(expr)), range)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    // Could be unknown, and deduced during type checking
    pub return_ty: Option<Type>,
    pub attributes: Attributes,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Expr>,
    pub attributes: Attributes,
}

impl Block {
    pub const fn empty() -> Self {
        Self {
            statements: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

pub mod utils {
    use crate::Ast;

    pub fn to_string(ast: &Ast) -> String {
        format!("{:#?}", ast)
    }
}
