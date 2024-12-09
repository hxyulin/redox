use std::ops::Range;

pub enum Literal {
    // TODO: add more
    Int(i64),
}

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

pub trait TopLevelTrait {
    fn is_top_level(&self) -> bool;
}

pub enum ExprKind {
    Literal(Literal),
}

pub enum TopLevelKind {
    Expr(Box<Expr>),
}

pub type Expr = Wrapped<ExprKind>;
pub type TopLevel = Wrapped<TopLevelKind>;
