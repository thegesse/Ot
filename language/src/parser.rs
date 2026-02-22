#![allow(unused)]
#![allow(deprecated)]
use chumsky::prelude::*;
use crate::lexer::Token;
use crate::ast::*;

pub fn parser() -> impl Parser<'static, &'static [Token], Vec<Statement>, extra::Err<Rich<'static, Token>>> {

    let expr = recursive(|expr| {
        let val = select! {
            Token::Integer(v) => Expression::LiteralInt(v),
            Token::Ident(name) => Expression::Variable(name),
            Token::StringLiteral(s) => Expression::LiteralString(s),
        };

        let op = select! {
            Token::Plus => Op::Add,
            Token::Minus => Op::Sub,
            Token::Multiply => Op::Mul,
            Token::Divide => Op::Div,
        };

        val.foldl(op.then(expr).repeated(), |lhs, (op, rhs)| {
            Expression::BinaryOp(Box::new(lhs), op, Box::new(rhs))
        })
    });

    let stmt = recursive(|stmt| {
        let manifestation = select! { Token::IntType => () }
            .then(select! { Token::Ident(name) => name })
            .then_ignore(select! { Token::Assignment => () })
            .then(expr.clone())
            .then_ignore(select! { Token::Semicolon => () })
            .map(|((_, name), value)| Statement::Manifestation {
                var_type: "int".to_string(),
                name,
                value,
            });

        // Output: Request_The_Universe_Manifest_The_Knowledge_Of x ...
        let print = select! { Token::Print => () }
            .then(expr.clone())
            .then_ignore(select! { Token::Semicolon => () })
            .map(|(_, e)| Statement::ManifestKnowledge(e));

        // The Loop: Initiate_The_Recursive_Protocol...
        let loop_stmt = select! { Token::For => () }
            .then(stmt.clone().map(Box::new))               // init (e.g., set i = 0)
            .then_ignore(select! { Token::ForCondition => () })
            .then(expr.clone())                             // condition (while i < 5)
            .then_ignore(select! { Token::ForStep => () })
            .then(stmt.clone().map(Box::new))               // step (i = i + 1)
            .then_ignore(select! { Token::LBrace => () })
            .then(stmt.clone().repeated().collect())        // body (inside braces)
            .then_ignore(select! { Token::RBrace => () })
            .map(|((((_, init), condition), step), body)| Statement::RecursiveProtocol {
                init,
                condition,
                step,
                body,
            });

        // The parser attempts to match the current token stream against these three
        manifestation.or(print).or(loop_stmt)
    });

    // Collect all repeated statements until the end of the input
    stmt.repeated().collect().then_ignore(end())
}