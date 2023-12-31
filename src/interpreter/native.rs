use std::fmt::Display;
use std::rc::Rc;
use super::env::EnvRef;
use super::types::{LoxCallable, LoxType};
use super::error::ErrorType;


fn clock(_: &[LoxType]) -> Result<LoxType, ErrorType> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    Ok(LoxType::Number(epoch.as_secs_f64()))
}

fn print(args: &[LoxType]) -> Result<LoxType, ErrorType> {
    let args: Vec<_> = args.iter().map(|arg| arg.to_string()).collect();
    println!("{}", args.join(" "));

    Ok(LoxType::Nil)
}


pub enum LoxFnNative {
    Clock,
    Print,
}

impl LoxCallable for LoxFnNative {
    fn call(&self, args: &[LoxType]) -> Result<LoxType, ErrorType> {
        match self {
            LoxFnNative::Clock => clock(args),
            LoxFnNative::Print => print(args),
        }
    }
}

impl Display for LoxFnNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[native fn]")
    }
}

pub fn populate(env: EnvRef) {
    let mut env = env.borrow_mut();
    env.define("clock", &LoxType::Callable(Rc::new(LoxFnNative::Clock)));
    env.define("print", &LoxType::Callable(Rc::new(LoxFnNative::Print)));
}
