// BOLD runtime: executes the validated AST.
// Phase 4 of the bld compiler pipeline (tree-walking runtime, v1).

use crate::parser::{BinOp, Expr, FnDef, Program, Stmt, UnOp};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Struct(String, HashMap<String, Value>),
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => {
                if v.fract() == 0.0 && v.abs() < 1e15 {
                    write!(f, "{:.1}", v)
                } else {
                    write!(f, "{}", v)
                }
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Struct(name, fields) => {
                let mut parts: Vec<String> =
                    fields.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                parts.sort();
                write!(f, "{} {{ {} }}", name, parts.join(", "))
            }
            Value::Unit => write!(f, "()"),
        }
    }
}

enum Flow {
    Normal(Value),
    Return(Value),
}

type RResult<T> = Result<T, (usize, String)>;

pub struct Runtime<'a> {
    fns: HashMap<String, &'a FnDef>,
}

struct Scope {
    vars: Vec<HashMap<String, Value>>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            vars: vec![HashMap::new()],
        }
    }
    fn push(&mut self) {
        self.vars.push(HashMap::new());
    }
    fn pop(&mut self) {
        self.vars.pop();
    }
    fn declare(&mut self, name: &str, v: Value) {
        self.vars.last_mut().unwrap().insert(name.to_string(), v);
    }
    fn set(&mut self, name: &str, v: Value) {
        for scope in self.vars.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), v);
                return;
            }
        }
    }
    fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.vars.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
}

impl<'a> Runtime<'a> {
    pub fn run(prog: &'a Program) -> RResult<()> {
        let mut fns = HashMap::new();
        for f in &prog.fns {
            fns.insert(f.name.clone(), f);
        }
        let rt = Runtime { fns };
        let main = rt
            .fns
            .get("main")
            .ok_or((1, "no main function. Every BOLD program starts at fn main()".to_string()))?;
        let mut scope = Scope::new();
        rt.exec_block(&main.body, &mut scope)?;
        Ok(())
    }

    fn exec_block(&self, stmts: &[Stmt], scope: &mut Scope) -> RResult<Flow> {
        for stmt in stmts {
            match stmt {
                Stmt::Let { name, expr, .. } => {
                    let v = self.eval(expr, scope)?;
                    scope.declare(name, v);
                }
                Stmt::Assign { name, expr, .. } => {
                    let v = self.eval(expr, scope)?;
                    scope.set(name, v);
                }
                Stmt::If { branches, els, .. } => {
                    let mut ran = false;
                    for (cond, body) in branches {
                        if let Value::Bool(true) = self.eval(cond, scope)? {
                            scope.push();
                            let flow = self.exec_block(body, scope)?;
                            scope.pop();
                            if let Flow::Return(v) = flow {
                                return Ok(Flow::Return(v));
                            }
                            ran = true;
                            break;
                        }
                    }
                    if !ran {
                        if let Some(body) = els {
                            scope.push();
                            let flow = self.exec_block(body, scope)?;
                            scope.pop();
                            if let Flow::Return(v) = flow {
                                return Ok(Flow::Return(v));
                            }
                        }
                    }
                }
                Stmt::For {
                    var,
                    start,
                    end,
                    body,
                    line,
                } => {
                    let s = match self.eval(start, scope)? {
                        Value::Int(v) => v,
                        _ => return Err((*line, "range start must be Int".into())),
                    };
                    let e = match self.eval(end, scope)? {
                        Value::Int(v) => v,
                        _ => return Err((*line, "range end must be Int".into())),
                    };
                    for i in s..e {
                        scope.push();
                        scope.declare(var, Value::Int(i));
                        let flow = self.exec_block(body, scope)?;
                        scope.pop();
                        if let Flow::Return(v) = flow {
                            return Ok(Flow::Return(v));
                        }
                    }
                }
                Stmt::While { cond, body, .. } => {
                    while let Value::Bool(true) = self.eval(cond, scope)? {
                        scope.push();
                        let flow = self.exec_block(body, scope)?;
                        scope.pop();
                        if let Flow::Return(v) = flow {
                            return Ok(Flow::Return(v));
                        }
                    }
                }
                Stmt::Return { expr, .. } => {
                    let v = match expr {
                        Some(e) => self.eval(e, scope)?,
                        None => Value::Unit,
                    };
                    return Ok(Flow::Return(v));
                }
                Stmt::ExprStmt(e) => {
                    self.eval(e, scope)?;
                }
            }
        }
        Ok(Flow::Normal(Value::Unit))
    }

    fn eval(&self, e: &Expr, scope: &mut Scope) -> RResult<Value> {
        match e {
            Expr::Int(v, _) => Ok(Value::Int(*v)),
            Expr::Float(v, _) => Ok(Value::Float(*v)),
            Expr::Str(s, _) => Ok(Value::Str(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Var(name, line) => scope
                .get(name)
                .cloned()
                .ok_or((*line, format!("unknown variable {}", name))),
            Expr::Unary(op, inner, line) => {
                let v = self.eval(inner, scope)?;
                match (op, v) {
                    (UnOp::Neg, Value::Int(x)) => Ok(Value::Int(-x)),
                    (UnOp::Neg, Value::Float(x)) => Ok(Value::Float(-x)),
                    (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err((*line, "invalid unary operation".into())),
                }
            }
            Expr::Bin(op, l, r, line) => {
                // short-circuit logic first
                if *op == BinOp::And {
                    return match self.eval(l, scope)? {
                        Value::Bool(false) => Ok(Value::Bool(false)),
                        _ => self.eval(r, scope),
                    };
                }
                if *op == BinOp::Or {
                    return match self.eval(l, scope)? {
                        Value::Bool(true) => Ok(Value::Bool(true)),
                        _ => self.eval(r, scope),
                    };
                }
                let lv = self.eval(l, scope)?;
                let rv = self.eval(r, scope)?;
                self.binop(*op, lv, rv, *line)
            }
            Expr::Call(name, args, line) => {
                let f = self
                    .fns
                    .get(name)
                    .ok_or((*line, format!("unknown function {}", name)))?;
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval(a, scope)?);
                }
                let mut inner = Scope::new();
                for ((pname, _), v) in f.params.iter().zip(vals) {
                    inner.declare(pname, v);
                }
                match self.exec_block(&f.body, &mut inner)? {
                    Flow::Return(v) => Ok(v),
                    Flow::Normal(_) => Ok(Value::Unit),
                }
            }
            Expr::ModCall(module, method, args, line) => {
                if module == "system" && method == "print" {
                    let v = self.eval(&args[0], scope)?;
                    println!("{}", v);
                    Ok(Value::Unit)
                } else {
                    Err((*line, format!("unknown module function {}.{}", module, method)))
                }
            }
            Expr::Field(base, field, line) => {
                let b = self.eval(base, scope)?;
                match b {
                    Value::Struct(sname, fields) => fields.get(field).cloned().ok_or((
                        *line,
                        format!("struct {} has no field named {}", sname, field),
                    )),
                    _ => Err((*line, format!("cannot access field {} on a non-struct", field))),
                }
            }
            Expr::StructLit(name, fields, _) => {
                let mut map = HashMap::new();
                for (fname, fexpr) in fields {
                    let v = self.eval(fexpr, scope)?;
                    map.insert(fname.clone(), v);
                }
                Ok(Value::Struct(name.clone(), map))
            }
        }
    }

    fn binop(&self, op: BinOp, l: Value, r: Value, line: usize) -> RResult<Value> {
        use BinOp::*;
        use Value::*;
        let v = match (op, &l, &r) {
            (Add, Int(a), Int(b)) => Int(a + b),
            (Add, Float(a), Float(b)) => Float(a + b),
            (Add, Str(a), Str(b)) => Str(format!("{}{}", a, b)),
            (Sub, Int(a), Int(b)) => Int(a - b),
            (Sub, Float(a), Float(b)) => Float(a - b),
            (Mul, Int(a), Int(b)) => Int(a * b),
            (Mul, Float(a), Float(b)) => Float(a * b),
            (Div, Int(_, ..), Int(0)) => return Err((line, "division by zero".into())),
            (Div, Int(a), Int(b)) => Int(a / b),
            (Div, Float(a), Float(b)) => Float(a / b),
            (Mod, Int(_, ..), Int(0)) => return Err((line, "modulo by zero".into())),
            (Mod, Int(a), Int(b)) => Int(a % b),
            (Lt, Int(a), Int(b)) => Bool(a < b),
            (Lt, Float(a), Float(b)) => Bool(a < b),
            (Gt, Int(a), Int(b)) => Bool(a > b),
            (Gt, Float(a), Float(b)) => Bool(a > b),
            (Le, Int(a), Int(b)) => Bool(a <= b),
            (Le, Float(a), Float(b)) => Bool(a <= b),
            (Ge, Int(a), Int(b)) => Bool(a >= b),
            (Ge, Float(a), Float(b)) => Bool(a >= b),
            (Eq, a, b) => Bool(value_eq(a, b)),
            (Ne, a, b) => Bool(!value_eq(a, b)),
            _ => return Err((line, "invalid binary operation".into())),
        };
        Ok(v)
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Struct(n1, f1), Value::Struct(n2, f2)) => {
            n1 == n2
                && f1.len() == f2.len()
                && f1.iter().all(|(k, v)| f2.get(k).map_or(false, |v2| value_eq(v, v2)))
        }
        (Value::Unit, Value::Unit) => true,
        _ => false,
    }
}
