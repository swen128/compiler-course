use super::ast::{self, Binding, Variable};
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

fn parse_literal(atom: &Atom, _position: Position) -> Result<ast::Expr, AstPasringError> {
    match atom {
        Atom::Integer(n) => Ok(ast::Expr::Lit(ast::Lit::Int(*n))),
        Atom::Boolean(b) => Ok(ast::Expr::Lit(ast::Lit::Bool(*b))),
        Atom::Character(c) => Ok(ast::Expr::Lit(ast::Lit::Char(*c))),
        Atom::String(s) => Ok(ast::Expr::String(s.to_owned())),
        Atom::Symbol(s) => match s.as_str() {
            "eof" => Ok(ast::Expr::Eof),
            _ => Ok(ast::Expr::Variable(Variable::new(s))),
        },
    }
}

fn parse_list(list: &List, _position: Position) -> Result<ast::Expr, AstPasringError> {
    let List(elems) = list;
    let mut elems = elems.iter();
    let head = match elems.next() {
        Some(expr) => expr,
        None => return Ok(ast::Expr::Lit(ast::Lit::EmptyList)),
    };
    let position = head.position.clone();

    match &head.kind {
        ExprKind::Atom(Atom::Symbol(s)) => match s.as_str() {
            "read-byte" => parse_prim0(ast::Op0::ReadByte, position, &mut elems),
            "peek-byte" => parse_prim0(ast::Op0::PeekByte, position, &mut elems),

            "add1" => parse_prim1(ast::Op1::Add1, position, &mut elems),
            "sub1" => parse_prim1(ast::Op1::Sub1, position, &mut elems),

            "zero?" => parse_prim1(ast::Op1::IsZero, position, &mut elems),
            "char?" => parse_prim1(ast::Op1::IsChar, position, &mut elems),
            "eof-object?" => parse_prim1(ast::Op1::IsEof, position, &mut elems),
            "box?" => parse_prim1(ast::Op1::IsBox, position, &mut elems),
            "cons?" => parse_prim1(ast::Op1::IsCons, position, &mut elems),
            "vector?" => parse_prim1(ast::Op1::IsVector, position, &mut elems),
            "string?" => parse_prim1(ast::Op1::IsString, position, &mut elems),

            "integer->char" => parse_prim1(ast::Op1::IntToChar, position, &mut elems),
            "char->integer" => parse_prim1(ast::Op1::CharToInt, position, &mut elems),

            "write-byte" => parse_prim1(ast::Op1::WriteByte, position, &mut elems),

            "box" => parse_prim1(ast::Op1::Box, position, &mut elems),
            "unbox" => parse_prim1(ast::Op1::Unbox, position, &mut elems),
            "car" => parse_prim1(ast::Op1::Car, position, &mut elems),
            "cdr" => parse_prim1(ast::Op1::Cdr, position, &mut elems),

            "+" => parse_prim2(ast::Op2::Add, position, &mut elems),
            "-" => parse_prim2(ast::Op2::Sub, position, &mut elems),
            "<" => parse_prim2(ast::Op2::LessThan, position, &mut elems),
            "=" => parse_prim2(ast::Op2::Equal, position, &mut elems),

            "cons" => parse_prim2(ast::Op2::Cons, position, &mut elems),
            "make-vector" => parse_prim2(ast::Op2::MakeVector, position, &mut elems),
            "make-string" => parse_prim2(ast::Op2::MakeString, position, &mut elems),
            "vector-ref" => parse_prim2(ast::Op2::VectorRef, position, &mut elems),
            "string-ref" => parse_prim2(ast::Op2::StringRef, position, &mut elems),

            "vector-set!" => parse_prim3(ast::Op3::VectorSet, position, &mut elems),

            "begin" => parse_begin(&mut elems, position),
            "if" => parse_if(&mut elems, position),
            "let" => parse_let(&mut elems, position),

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

fn parse_prim0<'a>(
    operator: ast::Op0,
    _position: Position,
    rest: &mut impl Iterator<Item = &'a Expr>,
) -> Result<ast::Expr, AstPasringError> {
    match rest.next() {
        None => Ok(ast::Expr::Prim0(operator)),
        Some(expr) => Err(err(
            "Expected 0 arguments. Got at least 1.",
            expr.position.clone(),
        )),
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

fn parse_prim2<'a>(
    operator: ast::Op2,
    position: Position,
    rest: &mut impl Iterator<Item = &'a Expr>,
) -> Result<ast::Expr, AstPasringError> {
    let operand_1 = rest
        .next()
        .ok_or(err("Missing first operand", position.clone()))?;
    let operand_1 = parse_expr(&operand_1)?;

    let operand_2 = rest
        .next()
        .ok_or(err("Missing second operand", position.clone()))?;
    let operand_2 = parse_expr(&operand_2)?;

    match rest.next() {
        None => Ok(ast::Expr::Prim2(
            operator,
            Box::new(operand_1),
            Box::new(operand_2),
        )),
        Some(expr) => Err(err(
            "Got more than 2 arguments for a binary operator.",
            expr.position.clone(),
        )),
    }
}

fn parse_prim3<'a>(
    operator: ast::Op3,
    position: Position,
    rest: &mut impl Iterator<Item = &'a Expr>,
) -> Result<ast::Expr, AstPasringError> {
    let operand_1 = rest
        .next()
        .ok_or(err("Missing first operand", position.clone()))?;
    let operand_1 = parse_expr(&operand_1)?;

    let operand_2 = rest
        .next()
        .ok_or(err("Missing second operand", position.clone()))?;
    let operand_2 = parse_expr(&operand_2)?;

    let operand_3 = rest
        .next()
        .ok_or(err("Missing third operand", position.clone()))?;
    let operand_3 = parse_expr(&operand_3)?;

    match rest.next() {
        None => Ok(ast::Expr::Prim3(
            operator,
            Box::new(operand_1),
            Box::new(operand_2),
            Box::new(operand_3),
        )),
        Some(expr) => Err(err(
            "Got more than 3 arguments for a ternary operator.",
            expr.position.clone(),
        )),
    }
}

fn parse_begin<'a>(
    rest: &mut impl Iterator<Item = &'a Expr>,
    position: Position,
) -> Result<ast::Expr, AstPasringError> {
    let first = rest
        .next()
        .ok_or(err("Missing first argument", position.clone()))?;
    let first = parse_expr(&first)?;

    let second = rest
        .next()
        .ok_or(err("Missing second argument", position.clone()))?;
    let second = parse_expr(&second)?;

    match rest.next() {
        None => Ok(ast::Expr::Begin(Box::new(first), Box::new(second))),
        Some(expr) => Err(err(
            "Expected 2 arguments. Got at least 3.",
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

fn parse_let<'a>(
    rest: &mut impl Iterator<Item = &'a Expr>,
    position: Position,
) -> Result<ast::Expr, AstPasringError> {
    let bindings = match rest.next() {
        None => Err(err("Missing variable bindings", position.clone())),

        Some(expr) => match &expr.kind {
            ExprKind::List(list) => parse_variable_bindings(&list, expr.position.clone()),
            _ => Err(err("Expected a list of bindings", expr.position.clone())),
        },
    }?;

    let binding = if bindings.len() == 1 {
        bindings.into_iter().next().unwrap()
    } else {
        return Err(err("Expected a single variable binding", position.clone()));
    };

    let body = rest
        .next()
        .ok_or(err("Missing body of `let` expression", position))?;

    match rest.next() {
        None => Ok(ast::Expr::Let(ast::Let {
            binding,
            body: Box::new(parse_expr(&body)?),
        })),
        Some(expr) => Err(err(
            "`let` expression should be of the form `(let <bindings> <body>)`, but got more than 2 arguments.",
            expr.position.clone(),
        )),
    }
}

/// # Arguments
/// * `list` - A list of variable bindings
fn parse_variable_bindings<'a>(
    list: &List,
    _position: Position,
) -> Result<Vec<Binding>, AstPasringError> {
    let List(elems) = list;
    let elems = elems.iter();

    elems
        .map(|expr| match &expr.kind {
            ExprKind::List(list) => parse_variable_binding(&list, expr.position.clone()),

            _ => Err(err("Expected a variable binding", expr.position.clone())),
        })
        .collect()
}

/// # Arguments
/// * `list` - A tuple of a variable name and a variable value
fn parse_variable_binding<'a>(list: &List, position: Position) -> Result<Binding, AstPasringError> {
    let List(elems) = list;
    let mut elems = elems.iter();

    let lhs = match elems.next() {
        Some(Expr {
            kind: ExprKind::Atom(Atom::Symbol(s)),
            position: _,
        }) => Variable(s.to_owned()),

        Some(expr) => return Err(err("Expected a variable name", expr.position.clone())),
        None => return Err(err("Missing variable name", position.clone())),
    };

    let rhs = elems
        .next()
        .ok_or(err("Missing variable value", position.clone()))?;
    let rhs = parse_expr(&rhs)?;

    match elems.next() {
        None => Ok(Binding {
            lhs,
            rhs: Box::new(rhs),
        }),
        Some(expr) => Err(err(
            "Expected 2 arguments. Got at least 3.",
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
