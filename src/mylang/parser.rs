use super::ast;
use super::s_expression::{Atom, Expr};

pub fn parse(expr: Expr) -> Result<ast::Program, String> {
    let ast_expr = parse_expr(expr);
    ast_expr.map(|expr| ast::Program { expr })
}

pub fn parse_expr(expr: Expr) -> Result<ast::Expr, String> {
    match expr {
        Expr::Atom(atom) => parse_literal(atom),
        Expr::List(list) => parse_list(list),
    }
}

fn parse_literal(atom: Atom) -> Result<ast::Expr, String> {
    match atom {
        Atom::Integer(n) => Ok(ast::Expr::Lit(ast::Lit::Int(n))),
        _ => Err("Expected literal".to_string()),
    }
}

fn parse_list(list: Vec<Expr>) -> Result<ast::Expr, String> {
    let mut list = list.into_iter();
    let op = list.next().ok_or("Empty list")?;
    let op = match op {
        Expr::Atom(atom) => match atom {
            Atom::Symbol(s) => match s.as_str() {
                "add1" => ast::Op1::Add1,
                "sub1" => ast::Op1::Sub1,
                _ => return Err("Unknown operator".to_string()),
            },
            _ => return Err("Expected operator".to_string()),
        },
        _ => return Err("Expected operator".to_string()),
    };
    let expr = list.next().ok_or("Missing operand")?;
    let expr = parse_expr(expr)?;
    Ok(ast::Expr::Prim1(op, Box::new(expr)))
}
