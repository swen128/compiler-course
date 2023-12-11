use super::ast;
use super::s_expression::{Atom, Expr, List};

pub fn parse(expr: &Expr) -> Result<ast::Program, String> {
    let ast_expr = parse_expr(expr);
    ast_expr.map(|expr| ast::Program { expr })
}

pub fn parse_expr(expr: &Expr) -> Result<ast::Expr, String> {
    match expr {
        Expr::Atom(atom) => parse_literal(atom),
        Expr::List(list) => parse_list(list),
    }
}

fn parse_literal(atom: &Atom) -> Result<ast::Expr, String> {
    match atom {
        Atom::Integer(n) => Ok(ast::Expr::Lit(ast::Lit::Int(*n))),
        Atom::Boolean(b) => Ok(ast::Expr::Lit(ast::Lit::Bool(*b))),
        Atom::Symbol(_) => Err(format!("Expected an integer or boolean literal. Got {:?}", atom)),
    }
}

fn parse_list(list: &List) -> Result<ast::Expr, String> {
    let head = &list.head;
    let mut tail = list.tail.iter();

    match head {
        Atom::Symbol(s) => match s.as_str() {
            "add1" => parse_prim1(ast::Op1::Add1, &mut tail),
            "sub1" => parse_prim1(ast::Op1::Sub1, &mut tail),
            "zero?" => parse_prim1(ast::Op1::IsZero, &mut tail),
            "if" => parse_if(&mut tail),
            _ => Err(format!("Unknown operator: {}", s)),
        },
        _ => Err("Expected operator".to_string()),
    }
}

fn parse_prim1<'a>(
    operator: ast::Op1,
    rest: &mut impl Iterator<Item = &'a Expr>, // TODO: I'm not sure why this lifetime annotation is required.
) -> Result<ast::Expr, String> {
    let operand = rest.next().ok_or("Missing operand")?;
    let operand = parse_expr(&operand)?;
    match rest.next() {
        None => Ok(ast::Expr::Prim1(operator, Box::new(operand))),
        Some(_) => Err("Expected 1 argument. Got at least 2.".to_string()),
    }
}

fn parse_if<'a>(rest: &mut impl Iterator<Item = &'a Expr>) -> Result<ast::Expr, String> {
    let cond = rest.next().ok_or("Missing condition")?;
    let cond = parse_expr(&cond)?;

    let then = rest.next().ok_or("Missing then")?;
    let then = parse_expr(&then)?;

    let els = rest.next().ok_or("Missing else")?;
    let els = parse_expr(&els)?;

    match rest.next() {
        None => Ok(ast::Expr::If(ast::If {
            cond: Box::new(cond),
            then: Box::new(then),
            els: Box::new(els),
        })),
        Some(_) => Err("Expected 3 arguments. Got at least 4.".to_string()),
    }
}
