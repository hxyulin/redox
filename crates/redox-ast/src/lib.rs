use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    // TODO: add more, and differentiate bit widths
    Int(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Tuple(Vec<Type>),
}

impl Type {
    pub fn empty() -> Self {
        // Ensure it doesn't allocate
        Type::Tuple(Vec::with_capacity(0))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Type::Tuple(types) => types.is_empty(),
        }
    }
}

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

pub trait IsTopLevel {
    fn is_top_level(&self) -> bool;
}

impl IsTopLevel for ExprKind {
    fn is_top_level(&self) -> bool {
        matches!(self, ExprKind::FunctionDef { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    FunctionDef { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelKind {
    Expr(Box<Expr>),
}

pub type Expr = Wrapped<ExprKind>;
pub type TopLevel = Wrapped<TopLevelKind>;

impl TopLevel {
    pub fn expr(expr: Expr) -> Self {
        let range = expr.span.clone();
        Self::new(TopLevelKind::Expr(Box::new(expr)), range)
    }
}
