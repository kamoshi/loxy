#[derive(Debug)]
pub enum Stmt {
    Var(Ident, Option<Box<Expr>>),
    Print(Box<Expr>),
    Expression(Box<Expr>),
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    Unary(OpUnary, Box<Expr>),
    Binary(Box<Expr>, OpBinary, Box<Expr>),
    Grouping(Box<Expr>),
    Variable(Ident),
}

#[derive(Debug)]
pub enum OpUnary {
    Not, Neg
}

#[derive(Debug)]
pub enum OpBinary {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Literal {
    Num(f64),
    Str(String),
    True,
    False,
    Nil,
}

#[derive(Debug)]
pub struct Ident(pub String);
