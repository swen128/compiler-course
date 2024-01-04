use super::ast;
use super::document::Position;
use super::error::AstPasringError;
use super::s_expression::{Atom, Expr, ExprKind, List};

type Result<T> = std::result::Result<T, super::error::AstPasringError>;

pub fn parse(s_expressions: &Vec<Expr>) -> Result<ast::Program> {
    let (last, rest) = s_expressions
        .split_last()
        .ok_or(err("Empty program.", Position::zero()))?;

    let function_definitions = rest
        .iter()
        .map(|expr| parse_function_definition(expr))
        .collect::<Result<Vec<_>>>()?;
    let expr = parse_expr(last)?;

    Ok(ast::Program {
        function_definitions,
        expr,
    })
}

pub fn parse_expr(expr: &Expr) -> Result<ast::Expr> {
    match &expr.kind {
        ExprKind::Atom(atom) => parse_literal(&atom),
        ExprKind::List(list) => parse_list(&list),
    }
}

fn parse_literal(atom: &Atom) -> Result<ast::Expr> {
    match atom {
        Atom::Integer(n) => Ok(ast::Expr::Lit(ast::Lit::Int(*n))),
        Atom::Boolean(b) => Ok(ast::Expr::Lit(ast::Lit::Bool(*b))),
        Atom::Character(c) => Ok(ast::Expr::Lit(ast::Lit::Char(*c))),
        Atom::String(s) => Ok(ast::Expr::Lit(ast::Lit::String(s.to_owned()))),
        Atom::Symbol(s) => match s.as_str() {
            "eof" => Ok(ast::Expr::Eof),
            _ => Ok(ast::Expr::Variable(ast::Identifier::new(s))),
        },
    }
}

fn parse_list(List(elems): &List) -> Result<ast::Expr> {
    match elems.as_slice() {
        [] => Ok(ast::Expr::Lit(ast::Lit::EmptyList)),

        [head, rest @ ..] => {
            let position = head.position.clone();

            match &head.kind {
                ExprKind::Atom(Atom::Symbol(s)) => match s.as_str() {
                    "read-byte" => parse_prim0(ast::Op0::ReadByte, position, rest),
                    "peek-byte" => parse_prim0(ast::Op0::PeekByte, position, rest),

                    "add1" => parse_prim1(ast::Op1::Add1, position, rest),
                    "sub1" => parse_prim1(ast::Op1::Sub1, position, rest),

                    "zero?" => parse_prim1(ast::Op1::IsZero, position, rest),
                    "char?" => parse_prim1(ast::Op1::IsChar, position, rest),
                    "eof-object?" => parse_prim1(ast::Op1::IsEof, position, rest),
                    "box?" => parse_prim1(ast::Op1::IsBox, position, rest),
                    "cons?" => parse_prim1(ast::Op1::IsCons, position, rest),
                    "vector?" => parse_prim1(ast::Op1::IsVector, position, rest),
                    "string?" => parse_prim1(ast::Op1::IsString, position, rest),

                    "integer->char" => parse_prim1(ast::Op1::IntToChar, position, rest),
                    "char->integer" => parse_prim1(ast::Op1::CharToInt, position, rest),

                    "write-byte" => parse_prim1(ast::Op1::WriteByte, position, rest),

                    "box" => parse_prim1(ast::Op1::Box, position, rest),
                    "unbox" => parse_prim1(ast::Op1::Unbox, position, rest),
                    "car" => parse_prim1(ast::Op1::Car, position, rest),
                    "cdr" => parse_prim1(ast::Op1::Cdr, position, rest),

                    "+" => parse_prim2(ast::Op2::Add, position, rest),
                    "-" => parse_prim2(ast::Op2::Sub, position, rest),
                    "<" => parse_prim2(ast::Op2::LessThan, position, rest),
                    "=" => parse_prim2(ast::Op2::Equal, position, rest),

                    "cons" => parse_prim2(ast::Op2::Cons, position, rest),
                    "make-vector" => parse_prim2(ast::Op2::MakeVector, position, rest),
                    "make-string" => parse_prim2(ast::Op2::MakeString, position, rest),
                    "vector-ref" => parse_prim2(ast::Op2::VectorRef, position, rest),
                    "string-ref" => parse_prim2(ast::Op2::StringRef, position, rest),

                    "vector-set!" => parse_prim3(ast::Op3::VectorSet, position, rest),

                    "begin" => parse_begin(rest, position),
                    "if" => parse_if(rest, position),
                    "let" => parse_let(rest, position),
                    "match" => parse_match(rest, position),

                    _ => parse_function_application(s.as_str(), rest, position),
                },

                _ => Err(err("The head of a list should be a symbol.", position)),
            }
        }
    }
}

fn parse_prim0<'a>(operator: ast::Op0, _position: Position, args: &[Expr]) -> Result<ast::Expr> {
    match args {
        [] => Ok(ast::Expr::Prim0(operator)),

        _ => {
            let msg = format!("The operator '{:?}' takes 0 arguments.", operator);
            Err(err(msg.as_str(), args[0].position.clone()))
        }
    }
}

fn parse_prim1<'a>(operator: ast::Op1, position: Position, args: &[Expr]) -> Result<ast::Expr> {
    match args {
        [arg] => Ok(ast::Expr::Prim1(operator, Box::new(parse_expr(arg)?))),

        _ => {
            let msg = format!("The operator '{:?}' takes 1 argument.", operator);
            Err(err(msg.as_str(), position))
        }
    }
}

fn parse_prim2<'a>(operator: ast::Op2, position: Position, args: &[Expr]) -> Result<ast::Expr> {
    match args {
        [arg_1, arg_2] => Ok(ast::Expr::Prim2(
            operator,
            Box::new(parse_expr(arg_1)?),
            Box::new(parse_expr(arg_2)?),
        )),

        _ => {
            let msg = format!("The operator '{:?}' takes 2 arguments.", operator);
            Err(err(msg.as_str(), position))
        }
    }
}

fn parse_prim3<'a>(operator: ast::Op3, position: Position, args: &[Expr]) -> Result<ast::Expr> {
    match args {
        [arg_1, arg_2, arg_3] => Ok(ast::Expr::Prim3(
            operator,
            Box::new(parse_expr(arg_1)?),
            Box::new(parse_expr(arg_2)?),
            Box::new(parse_expr(arg_3)?),
        )),

        _ => {
            let msg = format!("The operator '{:?}' takes 3 arguments.", operator);
            Err(err(msg.as_str(), position))
        }
    }
}

fn parse_begin<'a>(args: &[Expr], position: Position) -> Result<ast::Expr> {
    match args {
        [first, second] => Ok(ast::Expr::Begin(
            Box::new(parse_expr(first)?),
            Box::new(parse_expr(second)?),
        )),

        _ => {
            let msg = format!("The 'begin' expression takes 2 arguments.");
            Err(err(msg.as_str(), position))
        }
    }
}

fn parse_if<'a>(args: &[Expr], position: Position) -> Result<ast::Expr> {
    match args {
        [cond, then, els] => Ok(ast::Expr::If(ast::If {
            cond: Box::new(parse_expr(cond)?),
            then: Box::new(parse_expr(then)?),
            els: Box::new(parse_expr(els)?),
        })),

        _ => {
            let msg = format!("The 'if' expression takes 3 arguments.");
            Err(err(msg.as_str(), position))
        }
    }
}

fn parse_let<'a>(args: &[Expr], position: Position) -> Result<ast::Expr> {
    match args {
        [bindings, body] => Ok(ast::Expr::Let(ast::Let {
            binding: parse_variable_bindings(bindings)?,
            body: Box::new(parse_expr(body)?),
        })),
        _ => {
            let msg = format!("`let` expression should be of the form `(let <bindings> <body>)`");
            Err(err(msg.as_str(), position))
        }
    }
}

/// # Arguments
/// * `expr` - Should be a s-expression of the form `((<lhs> <rhs>))`.
fn parse_variable_bindings(expr: &Expr) -> Result<ast::Binding> {
    if let ExprKind::List(List(elems)) = &expr.kind {
        if let [binding] = elems.as_slice() {
            return parse_variable_binding(binding);
        }
    }
    Err(err(
        "Variable bindings should be of the form `((<lhs> <rhs>))`",
        expr.position.clone(),
    ))
}

/// # Arguments
/// * `list` - Should be a s-expression of the form `(<lhs> <rhs>)`.
fn parse_variable_binding(expr: &Expr) -> Result<ast::Binding> {
    if let ExprKind::List(List(elems)) = &expr.kind {
        if let [lhs, rhs] = elems.as_slice() {
            return Ok(ast::Binding {
                lhs: parse_identifier(lhs)?,
                rhs: Box::new(parse_expr(rhs)?),
            });
        }
    }
    Err(err(
        "Variable binding should be of the form `(<lhs> <rhs>)`",
        expr.position.clone(),
    ))
}

fn parse_match(args: &[Expr], position: Position) -> Result<ast::Expr> {
    match args {
        [expr, arms @ ..] => Ok(ast::Expr::Match(ast::Match {
            expr: Box::new(parse_expr(expr)?),
            arms: arms
                .iter()
                .map(parse_match_arm)
                .collect::<Result<Vec<_>>>()?,
        })),

        _ => {
            let msg =
                format!("The 'match' expression should be of the form `(match <expr> <arms>...)`");
            Err(err(msg.as_str(), position))
        }
    }
}

fn parse_match_arm(expr: &Expr) -> Result<ast::Arm> {
    if let ExprKind::List(List(elems)) = &expr.kind {
        if let [pattern, body] = elems.as_slice() {
            return Ok(ast::Arm {
                pattern: parse_pattern(pattern)?,
                body: Box::new(parse_expr(body)?),
            });
        }
    }
    Err(err(
        "Match arm should be of the form `(<pattern> <body>)`",
        expr.position.clone(),
    ))
}

fn parse_pattern(expr: &Expr) -> Result<ast::Pattern> {
    match &expr.kind {
        ExprKind::Atom(atom) => Ok(parse_atom_pattern(atom)),
        ExprKind::List(List(elems)) => parse_complex_pattern(elems),
    }
}

fn parse_atom_pattern(atom: &Atom) -> ast::Pattern {
    match atom {
        Atom::Integer(n) => ast::Pattern::Lit(ast::Lit::Int(*n)),
        Atom::Boolean(b) => ast::Pattern::Lit(ast::Lit::Bool(*b)),
        Atom::Character(c) => ast::Pattern::Lit(ast::Lit::Char(*c)),
        Atom::String(s) => ast::Pattern::Lit(ast::Lit::String(s.to_owned())),
        Atom::Symbol(s) => {
            if s == "_" {
                ast::Pattern::Wildcard
            } else {
                ast::Pattern::Variable(ast::Identifier::new(s))
            }
        }
    }
}

fn parse_complex_pattern(elems: &[Expr]) -> Result<ast::Pattern> {
    match elems {
        [] => Ok(ast::Pattern::Lit(ast::Lit::EmptyList)),

        [head, tail @ ..] => {
            if let ExprKind::Atom(Atom::Symbol(s)) = &head.kind {
                match (s.as_str(), tail) {
                    ("cons", [car, cdr]) => Ok(ast::Pattern::Cons(
                        Box::new(parse_pattern(car)?),
                        Box::new(parse_pattern(cdr)?),
                    )),
                    ("box", [expr]) => Ok(ast::Pattern::Box(Box::new(parse_pattern(expr)?))),
                    ("and", [left, right]) => Ok(ast::Pattern::And(
                        Box::new(parse_pattern(left)?),
                        Box::new(parse_pattern(right)?),
                    )),

                    _ => Err(err("Invalid pattern syntax.", head.position.clone())),
                }
            } else {
                Err(err("Invalid pattern syntax.", head.position.clone()))
            }
        }
    }
}

fn parse_function_application<'a>(
    function_name: &str,
    arguments: impl IntoIterator<Item = &'a Expr>,
    _position: Position,
) -> Result<ast::Expr> {
    let function = ast::Identifier(function_name.to_owned());
    let args = arguments
        .into_iter()
        .map(parse_expr)
        .collect::<Result<Vec<_>>>()?;
    Ok(ast::Expr::App(ast::App { function, args }))
}

/// Parse a function definition of the form: `(define (<name> <param> <param> ...) <body>)`
fn parse_function_definition(expr: &Expr) -> Result<ast::FunctionDefinition> {
    if let ExprKind::List(List(elems)) = &expr.kind {
        if let [define, signature, body] = elems.as_slice() {
            parse_define_keyword(define)?;
            let signature = parse_function_signature(signature)?;
            let body = parse_expr(&body)?;
            return Ok(ast::FunctionDefinition { signature, body });
        }
    }
    Err(err(
        "Function definition should be of the form `(define <signature> <body>)`",
        expr.position.clone(),
    ))
}

fn parse_define_keyword(expr: &Expr) -> Result<()> {
    match &expr.kind {
        ExprKind::Atom(Atom::Symbol(s)) if s == "define" => Ok(()),

        _ => Err(AstPasringError {
            msg: format!("Expected `define` keyword. Got {:?}", expr),
            position: expr.position.clone(),
        }),
    }
}

fn parse_function_signature(expr: &Expr) -> Result<ast::FunctionSignature> {
    if let ExprKind::List(List(elems)) = &expr.kind {
        if let [name, params @ ..] = elems.as_slice() {
            let name = parse_identifier(name)?;
            let params = params
                .iter()
                .map(parse_identifier)
                .collect::<Result<Vec<_>>>()?;

            return Ok(ast::FunctionSignature { name, params });
        }
    }
    Err(err(
        "Function signature should be of the form `(<name> <param> <param> ...)`",
        expr.position.clone(),
    ))
}

fn parse_identifier(expr: &Expr) -> Result<ast::Identifier> {
    match &expr.kind {
        ExprKind::Atom(Atom::Symbol(s)) => Ok(ast::Identifier::new(s)),
        _ => Err(AstPasringError {
            msg: format!("Expected an identifier. Got {:?}", expr),
            position: expr.position.clone(),
        }),
    }
}

fn err(msg: &str, position: Position) -> AstPasringError {
    AstPasringError {
        msg: msg.to_owned(),
        position,
    }
}
