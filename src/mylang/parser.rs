use super::ast;
use super::document::Position;
use super::error::AstPasringError;
use super::s_expression::{Atom, Expr, ExprKind, List};

pub fn parse(expr: &Expr) -> Result<ast::Program, AstPasringError> {
    let ast_expr = parse_expr(expr);
    ast_expr.map(|expr| ast::Program { expr })
}

pub fn parse_expr(expr: &Expr) -> Result<ast::Expr, AstPasringError> {
    let position = expr.position.clone();
    match &expr.kind {
        ExprKind::Atom(atom) => parse_literal(&atom, position),
        ExprKind::List(list) => parse_list(&list, position),
    }
}

fn parse_literal(atom: &Atom, position: Position) -> Result<ast::Expr, AstPasringError> {
    match atom {
        Atom::Integer(n) => Ok(ast::Expr::Lit(ast::Lit::Int(*n))),
        Atom::Boolean(b) => Ok(ast::Expr::Lit(ast::Lit::Bool(*b))),
        Atom::Character(c) => Ok(ast::Expr::Lit(ast::Lit::Char(*c))),
        Atom::Symbol(_) => Err(err(
            &format!("Expected a literal. Got a symbol: {:?}", atom),
            position.clone(),
        )),
    }
}

fn parse_list(list: &List, position: Position) -> Result<ast::Expr, AstPasringError> {
    let List(elems) = list;
    let mut elems = elems.iter();
    let head = elems.next().ok_or(err("Empty list.", position))?;
    let position = head.position.clone();

    match &head.kind {
        ExprKind::Atom(Atom::Symbol(s)) => match s.as_str() {
            "add1" => parse_prim1(ast::Op1::Add1, position, &mut elems),
            "sub1" => parse_prim1(ast::Op1::Sub1, position, &mut elems),
            "zero?" => parse_prim1(ast::Op1::IsZero, position, &mut elems),
            "char?" => parse_prim1(ast::Op1::IsChar, position, &mut elems),
            "integer->char" => parse_prim1(ast::Op1::IntToChar, position, &mut elems),
            "char->integer" => parse_prim1(ast::Op1::CharToInt, position, &mut elems),
            "if" => parse_if(&mut elems, position),
            _ => Err(AstPasringError {
                msg: format!("Unknown operator: {}", s),
                position,
            }),
        },
        _ => Err(AstPasringError {
            msg: "Expected an operator".to_owned(),
            position,
        }),
    }
}

fn parse_prim1<'a>(
    operator: ast::Op1,
    position: Position,
    rest: &mut impl Iterator<Item = &'a Expr>, // TODO: I'm not sure why this lifetime annotation is required.
) -> Result<ast::Expr, AstPasringError> {
    let operand = rest.next().ok_or(err("Missing operand", position))?;
    let operand = parse_expr(&operand)?;
    match rest.next() {
        None => Ok(ast::Expr::Prim1(operator, Box::new(operand))),
        Some(expr) => Err(err(
            "Expected 1 argument. Got at least 2.",
            expr.position.clone(),
        )),
    }
}

fn parse_if<'a>(
    rest: &mut impl Iterator<Item = &'a Expr>,
    position: Position,
) -> Result<ast::Expr, AstPasringError> {
    let cond = rest
        .next()
        .ok_or(err("Missing condition", position.clone()))?;
    let cond = parse_expr(&cond)?;

    let then = rest.next().ok_or(err("Missing 'then'", position.clone()))?;
    let then = parse_expr(&then)?;

    let els = rest.next().ok_or(err("Missing 'else'", position))?;
    let els = parse_expr(&els)?;

    match rest.next() {
        None => Ok(ast::Expr::If(ast::If {
            cond: Box::new(cond),
            then: Box::new(then),
            els: Box::new(els),
        })),
        Some(expr) => Err(err(
            "Expected 3 arguments. Got at least 4.",
            expr.position.clone(),
        )),
    }
}

fn err(msg: &str, position: Position) -> AstPasringError {
    AstPasringError {
        msg: msg.to_owned(),
        position,
    }
}
