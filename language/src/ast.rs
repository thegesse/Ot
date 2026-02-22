#[derive(Debug, Clone)]
pub enum Op {Add, Sub, Mul, Div}

#[derive(Debug, Clone)]
pub enum Expression {
    LiteralInt(i64),
    LiteralFloat(f64),
    LiteralString(String),
    Variable(String),
    BinaryOp(Box<Expression>, Op, Box<Expression>),
    FunctionCall(String, Vec<Expression>),
}


#[derive(Debug, Clone)]
pub enum Statement {
    Manifestation {var_type: String, name: String, value: Expression},

    ManifestKnowledge(Expression),

    RecursiveProtocol {
        init: Box<Statement>,
        condition: Expression,
        step: Box<Statement>,
        body: Vec<Statement>,
    },

    Conditional {
        if_block: (Expression, Vec<Statement>),
        elifs: Vec<(Expression, Vec<Statement>)>,
        else_block: Option<Vec<Statement>>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    StringLiteral(String),
    Var(String),
    BinaryOp {
        left: Box<Expr>,
        op: Op,
        right: Box<Expr>,
    },
}
