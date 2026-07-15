// BOLD type checker: enforces strict static typing at compile time.
// Phase 3 of the bld compiler pipeline.

use crate::parser::{BinOp, Expr, FnDef, Program, Stmt, StructDef, Type, UnOp};
use std::collections::HashMap;

type TResult<T> = Result<T, (usize, String)>;

struct Scope {
    vars: Vec<HashMap<String, (Type, bool)>>, // name -> (type, mutable)
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
    fn declare(&mut self, name: &str, ty: Type, mutable: bool) {
        self.vars
            .last_mut()
            .unwrap()
            .insert(name.to_string(), (ty, mutable));
    }
    fn lookup(&self, name: &str) -> Option<&(Type, bool)> {
        for scope in self.vars.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
}

pub struct Checker {
    structs: HashMap<String, Vec<(String, Type)>>,
    fns: HashMap<String, (Vec<Type>, Type)>,
    imports: Vec<String>,
}

impl Checker {
    pub fn check(prog: &Program) -> TResult<()> {
        let mut c = Checker {
            structs: HashMap::new(),
            fns: HashMap::new(),
            imports: prog.imports.clone(),
        };

        // pass 1: collect declarations
        for s in &prog.structs {
            if c.structs.contains_key(&s.name) {
                return Err((s.line, format!("struct {} is defined twice", s.name)));
            }
            c.structs.insert(s.name.clone(), s.fields.clone());
        }
        for s in &prog.structs {
            c.validate_struct(s)?;
        }
        for f in &prog.fns {
            if c.fns.contains_key(&f.name) {
                return Err((f.line, format!("function {} is defined twice", f.name)));
            }
            for (_, pty) in &f.params {
                c.validate_type(pty, f.line)?;
            }
            c.validate_type(&f.ret, f.line)?;
            c.fns.insert(
                f.name.clone(),
                (f.params.iter().map(|(_, t)| t.clone()).collect(), f.ret.clone()),
            );
        }

        // pass 2: check bodies
        for f in &prog.fns {
            c.check_fn(f)?;
        }
        Ok(())
    }

    fn validate_type(&self, ty: &Type, line: usize) -> TResult<()> {
        if let Type::Struct(name) = ty {
            if !self.structs.contains_key(name) {
                return Err((line, format!("unknown type {}", name)));
            }
        }
        Ok(())
    }

    fn validate_struct(&self, s: &StructDef) -> TResult<()> {
        for (fname, fty) in &s.fields {
            self.validate_type(fty, s.line)
                .map_err(|(l, m)| (l, format!("in struct {}, field {}: {}", s.name, fname, m)))?;
        }
        Ok(())
    }

    fn check_fn(&self, f: &FnDef) -> TResult<()> {
        let mut scope = Scope::new();
        for (pname, pty) in &f.params {
            scope.declare(pname, pty.clone(), false);
        }
        let returns = self.check_block(&f.body, &mut scope, &f.ret)?;
        if f.ret != Type::Unit && !returns {
            return Err((
                f.line,
                format!(
                    "function {} declares return type {} but does not return on every path",
                    f.name,
                    f.ret.name()
                ),
            ));
        }
        Ok(())
    }

    /// Returns true if the block definitely returns on every path.
    fn check_block(&self, stmts: &[Stmt], scope: &mut Scope, ret: &Type) -> TResult<bool> {
        let mut always_returns = false;
        for stmt in stmts {
            match stmt {
                Stmt::Let {
                    name,
                    ty,
                    expr,
                    mutable,
                    line,
                } => {
                    let ety = self.type_of(expr, scope)?;
                    if ety == Type::Unit {
                        return Err((*line, format!("cannot assign a Unit value to {}", name)));
                    }
                    let final_ty = match ty {
                        Some(annotated) => {
                            self.validate_type(annotated, *line)?;
                            if *annotated != ety {
                                return Err((
                                    *line,
                                    format!(
                                        "{} is declared as {} but assigned a {}",
                                        name,
                                        annotated.name(),
                                        ety.name()
                                    ),
                                ));
                            }
                            annotated.clone()
                        }
                        None => ety,
                    };
                    scope.declare(name, final_ty, *mutable);
                }
                Stmt::Assign { name, expr, line } => {
                    let (vty, mutable) = scope
                        .lookup(name)
                        .cloned()
                        .ok_or((*line, format!("unknown variable {}", name)))?;
                    if !mutable {
                        return Err((
                            *line,
                            format!(
                                "{} is immutable (declared with let). Declare it with mut to allow reassignment",
                                name
                            ),
                        ));
                    }
                    let ety = self.type_of(expr, scope)?;
                    if ety != vty {
                        return Err((
                            *line,
                            format!(
                                "cannot assign {} to {}, which is {}",
                                ety.name(),
                                name,
                                vty.name()
                            ),
                        ));
                    }
                }
                Stmt::If { branches, els, line: _ } => {
                    let mut all_return = true;
                    for (cond, body) in branches {
                        let cty = self.type_of(cond, scope)?;
                        if cty != Type::Bool {
                            return Err((
                                cond.line(),
                                format!("if condition must be Bool, found {}", cty.name()),
                            ));
                        }
                        scope.push();
                        let r = self.check_block(body, scope, ret)?;
                        scope.pop();
                        all_return = all_return && r;
                    }
                    match els {
                        Some(body) => {
                            scope.push();
                            let r = self.check_block(body, scope, ret)?;
                            scope.pop();
                            all_return = all_return && r;
                        }
                        None => all_return = false,
                    }
                    if all_return {
                        always_returns = true;
                    }
                }
                Stmt::For {
                    var,
                    start,
                    end,
                    body,
                    line,
                } => {
                    let sty = self.type_of(start, scope)?;
                    let ety = self.type_of(end, scope)?;
                    if sty != Type::Int || ety != Type::Int {
                        return Err((*line, "for range bounds must both be Int".into()));
                    }
                    scope.push();
                    scope.declare(var, Type::Int, false);
                    self.check_block(body, scope, ret)?;
                    scope.pop();
                }
                Stmt::While { cond, body, line: _ } => {
                    let cty = self.type_of(cond, scope)?;
                    if cty != Type::Bool {
                        return Err((
                            cond.line(),
                            format!("while condition must be Bool, found {}", cty.name()),
                        ));
                    }
                    scope.push();
                    self.check_block(body, scope, ret)?;
                    scope.pop();
                }
                Stmt::Return { expr, line } => {
                    let rty = match expr {
                        Some(e) => self.type_of(e, scope)?,
                        None => Type::Unit,
                    };
                    if rty != *ret {
                        return Err((
                            *line,
                            format!(
                                "return type mismatch: function returns {}, found {}",
                                ret.name(),
                                rty.name()
                            ),
                        ));
                    }
                    always_returns = true;
                }
                Stmt::ExprStmt(e) => {
                    self.type_of(e, scope)?;
                }
            }
        }
        Ok(always_returns)
    }

    fn type_of(&self, e: &Expr, scope: &Scope) -> TResult<Type> {
        match e {
            Expr::Int(_, _) => Ok(Type::Int),
            Expr::Float(_, _) => Ok(Type::Float),
            Expr::Str(_, _) => Ok(Type::Str),
            Expr::Bool(_, _) => Ok(Type::Bool),
            Expr::Var(name, line) => scope
                .lookup(name)
                .map(|(t, _)| t.clone())
                .ok_or((*line, format!("unknown variable {}", name))),
            Expr::Unary(op, inner, line) => {
                let t = self.type_of(inner, scope)?;
                match op {
                    UnOp::Neg => {
                        if t == Type::Int || t == Type::Float {
                            Ok(t)
                        } else {
                            Err((*line, format!("cannot negate a {}", t.name())))
                        }
                    }
                    UnOp::Not => {
                        if t == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err((*line, format!("! requires a Bool, found {}", t.name())))
                        }
                    }
                }
            }
            Expr::Bin(op, l, r, line) => {
                let lt = self.type_of(l, scope)?;
                let rt = self.type_of(r, scope)?;
                match op {
                    BinOp::Add => match (&lt, &rt) {
                        (Type::Int, Type::Int) => Ok(Type::Int),
                        (Type::Float, Type::Float) => Ok(Type::Float),
                        (Type::Str, Type::Str) => Ok(Type::Str),
                        _ => Err((
                            *line,
                            format!(
                                "cannot add {} and {}. BOLD is strictly typed and does not convert implicitly",
                                lt.name(),
                                rt.name()
                            ),
                        )),
                    },
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => match (&lt, &rt) {
                        (Type::Int, Type::Int) => Ok(Type::Int),
                        (Type::Float, Type::Float) => Ok(Type::Float),
                        _ => Err((
                            *line,
                            format!(
                                "arithmetic requires two Ints or two Floats, found {} and {}",
                                lt.name(),
                                rt.name()
                            ),
                        )),
                    },
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => match (&lt, &rt) {
                        (Type::Int, Type::Int) | (Type::Float, Type::Float) => Ok(Type::Bool),
                        _ => Err((
                            *line,
                            format!(
                                "comparison requires two Ints or two Floats, found {} and {}",
                                lt.name(),
                                rt.name()
                            ),
                        )),
                    },
                    BinOp::Eq | BinOp::Ne => {
                        if lt == rt {
                            Ok(Type::Bool)
                        } else {
                            Err((
                                *line,
                                format!("cannot compare {} with {}", lt.name(), rt.name()),
                            ))
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if lt == Type::Bool && rt == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err((*line, "&& and || require Bool operands".into()))
                        }
                    }
                }
            }
            Expr::Call(name, args, line) => {
                let (ptys, ret) = self
                    .fns
                    .get(name)
                    .cloned()
                    .ok_or((*line, format!("unknown function {}", name)))?;
                if args.len() != ptys.len() {
                    return Err((
                        *line,
                        format!(
                            "{} expects {} argument(s), found {}",
                            name,
                            ptys.len(),
                            args.len()
                        ),
                    ));
                }
                for (i, (a, pty)) in args.iter().zip(ptys.iter()).enumerate() {
                    let aty = self.type_of(a, scope)?;
                    if aty != *pty {
                        return Err((
                            a.line(),
                            format!(
                                "argument {} of {} expects {}, found {}",
                                i + 1,
                                name,
                                pty.name(),
                                aty.name()
                            ),
                        ));
                    }
                }
                Ok(ret)
            }
            Expr::ModCall(module, method, args, line) => {
                if module == "system" {
                    if !self.imports.iter().any(|i| i == "system") {
                        return Err((
                            *line,
                            "the system module is not imported. Add: import system".into(),
                        ));
                    }
                    match method.as_str() {
                        "print" => {
                            if args.len() != 1 {
                                return Err((
                                    *line,
                                    "system.print expects exactly one argument".into(),
                                ));
                            }
                            self.type_of(&args[0], scope)?;
                            Ok(Type::Unit)
                        }
                        other => Err((
                            *line,
                            format!("the system module has no function named {}", other),
                        )),
                    }
                } else {
                    Err((*line, format!("unknown module {}", module)))
                }
            }
            Expr::Field(base, field, line) => {
                let bty = self.type_of(base, scope)?;
                match bty {
                    Type::Struct(sname) => {
                        let fields = self
                            .structs
                            .get(&sname)
                            .ok_or((*line, format!("unknown struct {}", sname)))?;
                        fields
                            .iter()
                            .find(|(f, _)| f == field)
                            .map(|(_, t)| t.clone())
                            .ok_or((
                                *line,
                                format!("struct {} has no field named {}", sname, field),
                            ))
                    }
                    other => Err((
                        *line,
                        format!("{} is not a struct, cannot access field {}", other.name(), field),
                    )),
                }
            }
            Expr::StructLit(name, fields, line) => {
                let def = self
                    .structs
                    .get(name)
                    .cloned()
                    .ok_or((*line, format!("unknown struct {}", name)))?;
                if fields.len() != def.len() {
                    return Err((
                        *line,
                        format!(
                            "struct {} has {} field(s), literal provides {}",
                            name,
                            def.len(),
                            fields.len()
                        ),
                    ));
                }
                for (fname, fexpr) in fields {
                    let expected = def
                        .iter()
                        .find(|(n, _)| n == fname)
                        .map(|(_, t)| t.clone())
                        .ok_or((
                            fexpr.line(),
                            format!("struct {} has no field named {}", name, fname),
                        ))?;
                    let actual = self.type_of(fexpr, scope)?;
                    if actual != expected {
                        return Err((
                            fexpr.line(),
                            format!(
                                "field {} of {} expects {}, found {}",
                                fname,
                                name,
                                expected.name(),
                                actual.name()
                            ),
                        ));
                    }
                }
                Ok(Type::Struct(name.clone()))
            }
        }
    }
}
