use std::rc::Rc;

use crate::parser::ast::{Stmt, Expr, Literal, OpUnary, OpBinary, Ident, OpLogic};
use super::types::{LoxType, LoxFn};
use super::env::{Env, EnvRef};
use super::error::ErrorType;


pub fn exec(g_env: Option<EnvRef>, stmts: &[Stmt]) -> Result<(), ErrorType> {
    let mut env = match g_env {
        Some(env) => env,
        None => Env::new_ref(),
    };

    for stmt in stmts {
        // lexical scope fix
        // scheme style!
        if matches!(stmt, Stmt::Var(..)) { env = Env::wrap(env); }

        exec_stmt(env.clone(), stmt)?;
    };

    Ok(())
}

fn exec_stmt(env: EnvRef, stmt: &Stmt) -> Result<(), ErrorType> {
    match stmt {
        Stmt::Var(ident, expr)  => exec_stmt_var(env, ident, expr)?,
        Stmt::If(cond, t, f)    => exec_stmt_if(env, cond, t, f)?,
        Stmt::Expression(expr)  => exec_stmt_expr(env, expr)?,
        Stmt::Block(stmts)      => exec(Some(Env::wrap(env)), stmts)?,
        Stmt::While(cond, stmt) => exec_stmt_while(env, cond, stmt)?,
        Stmt::Function(n, p, b) => exec_stmt_func(env, n, p, b)?,
        Stmt::Return(expr)      => exec_stmt_return(env, expr)?,
    };
    Ok(())
}

fn exec_stmt_return(env: EnvRef, expr: &Option<Box<Expr>>) -> Result<(), ErrorType> {
    let res = match expr {
        Some(expr)  => eval_expr(env, expr)?,
        None        => LoxType::Nil,
    };

    Err(ErrorType::Return(res))
}

fn exec_stmt_func(env: EnvRef, n: &Ident, p: &[Ident], b: &[Stmt]) -> Result<(), ErrorType> {
    let params: Vec<_> = p.iter().map(|Ident(name)| name.to_owned()).collect();
    let func = LoxFn::new(params, b.to_vec(), env.clone());
    env.borrow_mut().define(&n.0, &LoxType::Callable(Rc::new(func)));

    Ok(())
}

fn exec_stmt_while(env: EnvRef, cond: &Expr, stmt: &Stmt) -> Result<(), ErrorType> {
    while eval_expr(env.clone(), cond)?.is_truthy() {
        exec_stmt(env.clone(), stmt)?;
    };
    Ok(())
}

fn exec_stmt_if(env: EnvRef, cond: &Expr, t: &Stmt, f: &Option<Box<Stmt>>) -> Result<(), ErrorType> {
    let cond = eval_expr(env.clone(), cond)?;

    use LoxType::*;
    match (cond, f) {
        (Boolean(true), _)  => exec_stmt(env, t),
        (_, Some(stmt))     => exec_stmt(env, &stmt),
        _                   => Ok(()),
    }

}

fn exec_stmt_var(env: EnvRef, ident: &Ident, expr: &Option<Box<Expr>>) -> Result<(), ErrorType> {
    let value = match expr {
        Some(expr)  => eval_expr(env.clone(), expr)?,
        None        => LoxType::Nil,
    };

    env.borrow_mut().define(&ident.0, &value);

    Ok(())
}

fn exec_stmt_expr(env: EnvRef, expr: &Expr) -> Result<(), ErrorType> {
    eval_expr(env, expr)?;
    Ok(())
}


pub fn eval_expr(env: EnvRef, expr: &Expr) -> Result<LoxType, ErrorType> {
    match expr {
        Expr::Literal(literal)      => Ok(eval_expr_literal(literal)),
        Expr::Unary(op, expr)       => eval_expr_unary(env, op, expr),
        Expr::Binary(l, op, r)      => eval_expr_binary(env, l, op, r),
        Expr::Grouping(expr)        => eval_expr_grouping(env, expr),
        Expr::Variable(ident)       => eval_expr_variable(env, ident),
        Expr::Assign(ident, expr)   => eval_expr_assign(env, ident, expr),
        Expr::Logic(l, op, r)       => eval_expr_logic(env, l, op, r),
        Expr::Call(callee, args)    => eval_expr_call(env, callee, args),
        Expr::Lambda(ident, block)  => eval_expr_lambda(env, ident, block),
    }
}

fn eval_expr_lambda(env: EnvRef, ident: &[Ident], block: &[Stmt]) -> Result<LoxType, ErrorType> {
    let params: Vec<_> = ident.iter().map(|i| i.0.clone()).collect();
    Ok(LoxType::Callable(Rc::new(LoxFn::new(params, block.to_vec(), env.clone()))))
}

fn eval_expr_call(env: EnvRef, callee: &Expr, args: &[Expr]) -> Result<LoxType, ErrorType> {
    let callee = eval_expr(env.clone(), callee)?;
    let args: Vec<_> = args.iter()
        .map(|a| eval_expr(env.clone(), a))
        .collect::<Result<_, ErrorType>>()?;

    let res = match callee {
        LoxType::Callable(c) => c.call(&args),
        _ => Err(ErrorType::TypeMismatch("Can't call this value"))
    };

    match res {
        Err(ErrorType::Return(res)) => Ok(res),
        other => other,
    }
}

fn eval_expr_logic(env: EnvRef, l: &Expr, op: &OpLogic, r: &Expr) -> Result<LoxType, ErrorType> {
    let l = eval_expr(env.clone(), l)?;

    match (l.is_truthy(), op) {
        (true, OpLogic::Or)     => Ok(l),
        (false, OpLogic::Or)    => Ok(eval_expr(env, r)?),
        (true, OpLogic::And)    => Ok(eval_expr(env, r)?),
        (false, OpLogic::And)   => Ok(l),
    }
}

fn eval_expr_variable(env: EnvRef, ident: &Ident) -> Result<LoxType, ErrorType> {
    env.borrow().get(&ident.0)
}

fn eval_expr_assign(env: EnvRef, ident: &Ident, expr: &Expr) -> Result<LoxType, ErrorType> {
    let value = eval_expr(env.clone(), expr)?;

    env.borrow_mut().set(&ident.0, &value)?;
    Ok(value)
}

fn eval_expr_literal(literal: &Literal) -> LoxType {
    use LoxType::*;
    match literal {
        Literal::Num(n) => Number(*n),
        Literal::Str(s) => String(s.to_owned()),
        Literal::True   => Boolean(true),
        Literal::False  => Boolean(false),
        Literal::Nil    => Nil,
    }
}

fn eval_expr_unary(env: EnvRef, op: &OpUnary, expr: &Box<Expr>) -> Result<LoxType, ErrorType> {
    let value = eval_expr(env, expr)?;

    use LoxType::*;
    match op {
        OpUnary::Not => Ok(LoxType::Boolean(!value.is_truthy())),
        OpUnary::Neg => match value {
            Nil         => Err(ErrorType::TypeMismatch("Can't negate a nil value")),
            Boolean(_)  => Err(ErrorType::TypeMismatch("Can't negate a boolean value")),
            Number(f)   => Ok(Number(-f)),
            String(_)   => Err(ErrorType::TypeMismatch("Can't negate a string value")),
            Callable(_) => Err(ErrorType::TypeMismatch("Can't negate a function value")),
        },
    }
}

fn eval_expr_binary(env: EnvRef, l: &Box<Expr>, op: &OpBinary, r: &Box<Expr>) -> Result<LoxType, ErrorType> {
    let l = eval_expr(env.clone(), l)?;
    let r = eval_expr(env.clone(), r)?;

    use OpBinary::*;
    use LoxType::*;
    match op {
        Equal => match (l, r) {
            (Nil, Nil)  => Ok(Boolean(true)),
            (l, r)      => Ok(Boolean(l == r)),
        },
        NotEqual => match (l, r) {
            (Nil, Nil)  => Ok(Boolean(false)),
            (l, r)      => Ok(Boolean(l != r)),
        },
        Less => match (l, r) {
            (Number(l), Number(r)) => Ok(Boolean(l < r)),
            _ => Err(ErrorType::TypeMismatch("Can't compare non numbers")),
        },
        LessEqual => match (l, r) {
            (Number(l), Number(r)) => Ok(Boolean(l <= r)),
            _ => Err(ErrorType::TypeMismatch("Can't compare non numbers")),
        },
        Greater => match (l, r) {
            (Number(l), Number(r)) => Ok(Boolean(l > r)),
            _ => Err(ErrorType::TypeMismatch("Can't compare non numbers")),
        },
        GreaterEqual => match (l, r) {
            (Number(l), Number(r)) => Ok(Boolean(l >= r)),
            _ => Err(ErrorType::TypeMismatch("Can't compare non numbers")),
        },
        Add => match (l, r) {
            (Number(l), Number(r)) => Ok(Number(l + r)),
            (String(l), String(r)) => Ok(String(format!("{l}{r}"))),
            _ => Err(ErrorType::TypeMismatch("Can only add two numbers or two strings")),
        },
        Sub => match (l, r) {
            (Number(l), Number(r)) => Ok(Number(l - r)),
            _ => Err(ErrorType::TypeMismatch("Can't sub non numbers")),
        },
        Mul => match (l, r) {
            (Number(l), Number(r)) => Ok(Number(l * r)),
            _ => Err(ErrorType::TypeMismatch("Can't mul non numbers")),
        },
        Div => match (l, r) {
            (Number(l), Number(r)) => Ok(Number(l / r)),
            _ => Err(ErrorType::TypeMismatch("Can't div non numbers")),
        },
    }
}

fn eval_expr_grouping(env: EnvRef, expr: &Box<Expr>) -> Result<LoxType, ErrorType> {
    eval_expr(env, expr)
}
