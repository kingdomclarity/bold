// BOLD parser: consumes tokens and produces the Abstract Syntax Tree.
// Phase 2 of the bld compiler pipeline.

use crate::lexer::{Tok, Token};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    Unit,
    Struct(String),
}

impl Type {
    pub fn from_name(name: &str) -> Type {
        match name {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "String" => Type::Str,
            "Bool" => Type::Bool,
            other => Type::Struct(other.to_string()),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Type::Int => "Int".into(),
            Type::Float => "Float".into(),
            Type::Str => "String".into(),
            Type::Bool => "Bool".into(),
            Type::Unit => "Unit".into(),
            Type::Struct(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64, usize),
    Float(f64, usize),
    Str(String, usize),
    Bool(bool, usize),
    Var(String, usize),
    Unary(UnOp, Box<Expr>, usize),
    Bin(BinOp, Box<Expr>, Box<Expr>, usize),
    Call(String, Vec<Expr>, usize),
    ModCall(String, String, Vec<Expr>, usize),
    Field(Box<Expr>, String, usize),
    StructLit(String, Vec<(String, Expr)>, usize),
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Int(_, l)
            | Expr::Float(_, l)
            | Expr::Str(_, l)
            | Expr::Bool(_, l)
            | Expr::Var(_, l)
            | Expr::Unary(_, _, l)
            | Expr::Bin(_, _, _, l)
            | Expr::Call(_, _, l)
            | Expr::ModCall(_, _, _, l)
            | Expr::Field(_, _, l)
            | Expr::StructLit(_, _, l) => *l,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Type>,
        expr: Expr,
        mutable: bool,
        line: usize,
    },
    Assign {
        name: String,
        expr: Expr,
        line: usize,
    },
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        els: Option<Vec<Stmt>>,
        line: usize,
    },
    For {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    Return {
        expr: Option<Expr>,
        line: usize,
    },
    ExprStmt(Expr),
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<String>,
    pub structs: Vec<StructDef>,
    pub fns: Vec<FnDef>,
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    struct_names: HashSet<String>,
}

type PResult<T> = Result<T, (usize, String)>;

impl Parser {
    pub fn new(toks: Vec<Token>) -> Parser {
        // Pre-scan for struct names so struct literals can be told apart
        // from block-opening braces (the `if x {` ambiguity).
        let mut struct_names = HashSet::new();
        for w in toks.windows(2) {
            if w[0].tok == Tok::Struct {
                if let Tok::Ident(n) = &w[1].tok {
                    struct_names.insert(n.clone());
                }
            }
        }
        Parser {
            toks,
            pos: 0,
            struct_names,
        }
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek2(&self) -> &Tok {
        if self.pos + 1 < self.toks.len() {
            &self.toks[self.pos + 1].tok
        } else {
            &Tok::Eof
        }
    }
    fn line(&self) -> usize {
        self.toks[self.pos].line
    }
    fn next(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn expect(&mut self, t: Tok, what: &str) -> PResult<()> {
        if *self.peek() == t {
            self.next();
            Ok(())
        } else {
            Err((self.line(), format!("expected {}, found {:?}", what, self.peek())))
        }
    }
    fn ident(&mut self, what: &str) -> PResult<String> {
        match self.peek().clone() {
            Tok::Ident(n) => {
                self.next();
                Ok(n)
            }
            other => Err((self.line(), format!("expected {}, found {:?}", what, other))),
        }
    }

    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut prog = Program {
            imports: Vec::new(),
            structs: Vec::new(),
            fns: Vec::new(),
        };
        loop {
            match self.peek().clone() {
                Tok::Eof => break,
                Tok::Import => {
                    self.next();
                    let name = self.ident("module name after import")?;
                    prog.imports.push(name);
                }
                Tok::Struct => {
                    let s = self.parse_struct()?;
                    prog.structs.push(s);
                }
                Tok::Fn => {
                    let f = self.parse_fn()?;
                    prog.fns.push(f);
                }
                other => {
                    return Err((
                        self.line(),
                        format!(
                            "expected fn, struct, or import at top level, found {:?}",
                            other
                        ),
                    ))
                }
            }
        }
        Ok(prog)
    }

    fn parse_type(&mut self) -> PResult<Type> {
        let n = self.ident("a type name")?;
        Ok(Type::from_name(&n))
    }

    fn parse_struct(&mut self) -> PResult<StructDef> {
        let line = self.line();
        self.expect(Tok::Struct, "struct")?;
        let name = self.ident("struct name")?;
        self.expect(Tok::LBrace, "{ after struct name")?;
        let mut fields = Vec::new();
        while *self.peek() != Tok::RBrace {
            let fname = self.ident("field name")?;
            self.expect(Tok::Colon, ": after field name")?;
            let fty = self.parse_type()?;
            fields.push((fname, fty));
            if *self.peek() == Tok::Comma {
                self.next();
            }
        }
        self.expect(Tok::RBrace, "} to close struct")?;
        Ok(StructDef { name, fields, line })
    }

    fn parse_fn(&mut self) -> PResult<FnDef> {
        let line = self.line();
        self.expect(Tok::Fn, "fn")?;
        let name = self.ident("function name")?;
        self.expect(Tok::LParen, "( after function name")?;
        let mut params = Vec::new();
        while *self.peek() != Tok::RParen {
            let pname = self.ident("parameter name")?;
            self.expect(Tok::Colon, ": after parameter name")?;
            let pty = self.parse_type()?;
            params.push((pname, pty));
            if *self.peek() == Tok::Comma {
                self.next();
            }
        }
        self.expect(Tok::RParen, ") to close parameters")?;
        let ret = if *self.peek() == Tok::Arrow {
            self.next();
            self.parse_type()?
        } else {
            Type::Unit
        };
        let body = self.parse_block()?;
        Ok(FnDef {
            name,
            params,
            ret,
            body,
            line,
        })
    }

    fn parse_block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect(Tok::LBrace, "{ to open a block")?;
        let mut stmts = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return Err((self.line(), "unexpected end of file inside a block, missing }".into()));
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Tok::RBrace, "} to close a block")?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Let | Tok::Mut => {
                let mutable = *self.peek() == Tok::Mut;
                self.next();
                let name = self.ident("variable name")?;
                let ty = if *self.peek() == Tok::Colon {
                    self.next();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(Tok::Assign, "= in declaration")?;
                let expr = self.parse_expr()?;
                Ok(Stmt::Let {
                    name,
                    ty,
                    expr,
                    mutable,
                    line,
                })
            }
            Tok::If => {
                self.next();
                let mut branches = Vec::new();
                let cond = self.parse_expr_no_struct()?;
                let body = self.parse_block()?;
                branches.push((cond, body));
                let mut els = None;
                while *self.peek() == Tok::Else {
                    self.next();
                    if *self.peek() == Tok::If {
                        self.next();
                        let c = self.parse_expr_no_struct()?;
                        let b = self.parse_block()?;
                        branches.push((c, b));
                    } else {
                        els = Some(self.parse_block()?);
                        break;
                    }
                }
                Ok(Stmt::If { branches, els, line })
            }
            Tok::For => {
                self.next();
                let var = self.ident("loop variable")?;
                self.expect(Tok::In, "in")?;
                let start = self.parse_range_bound()?;
                self.expect(Tok::DotDot, ".. in range")?;
                let end = self.parse_range_bound()?;
                let body = self.parse_block()?;
                Ok(Stmt::For {
                    var,
                    start,
                    end,
                    body,
                    line,
                })
            }
            Tok::While => {
                self.next();
                let cond = self.parse_expr_no_struct()?;
                let body = self.parse_block()?;
                Ok(Stmt::While { cond, body, line })
            }
            Tok::Return => {
                self.next();
                // return with no expression: next token starts a new statement or closes block
                let expr = match self.peek() {
                    Tok::RBrace | Tok::Let | Tok::Mut | Tok::If | Tok::For | Tok::While
                    | Tok::Return => None,
                    _ => Some(self.parse_expr()?),
                };
                Ok(Stmt::Return { expr, line })
            }
            Tok::Ident(name) => {
                // assignment or expression statement
                if *self.peek2() == Tok::Assign {
                    self.next();
                    self.next();
                    let expr = self.parse_expr()?;
                    Ok(Stmt::Assign { name, expr, line })
                } else {
                    let e = self.parse_expr()?;
                    Ok(Stmt::ExprStmt(e))
                }
            }
            _ => {
                let e = self.parse_expr()?;
                Ok(Stmt::ExprStmt(e))
            }
        }
    }

    // Range bounds are simple expressions without `..` handling.
    fn parse_range_bound(&mut self) -> PResult<Expr> {
        self.parse_additive(false)
    }

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_or(true)
    }
    // In `if` and `while` conditions a `{` always opens the block,
    // so struct literals are not allowed at the top level there.
    fn parse_expr_no_struct(&mut self) -> PResult<Expr> {
        self.parse_or(false)
    }

    fn parse_or(&mut self, sl: bool) -> PResult<Expr> {
        let mut left = self.parse_and(sl)?;
        while *self.peek() == Tok::OrOr {
            let line = self.line();
            self.next();
            let right = self.parse_and(sl)?;
            left = Expr::Bin(BinOp::Or, Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn parse_and(&mut self, sl: bool) -> PResult<Expr> {
        let mut left = self.parse_equality(sl)?;
        while *self.peek() == Tok::AndAnd {
            let line = self.line();
            self.next();
            let right = self.parse_equality(sl)?;
            left = Expr::Bin(BinOp::And, Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn parse_equality(&mut self, sl: bool) -> PResult<Expr> {
        let mut left = self.parse_comparison(sl)?;
        loop {
            let op = match self.peek() {
                Tok::Eq => BinOp::Eq,
                Tok::Ne => BinOp::Ne,
                _ => break,
            };
            let line = self.line();
            self.next();
            let right = self.parse_comparison(sl)?;
            left = Expr::Bin(op, Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn parse_comparison(&mut self, sl: bool) -> PResult<Expr> {
        let mut left = self.parse_additive(sl)?;
        loop {
            let op = match self.peek() {
                Tok::Lt => BinOp::Lt,
                Tok::Gt => BinOp::Gt,
                Tok::Le => BinOp::Le,
                Tok::Ge => BinOp::Ge,
                _ => break,
            };
            let line = self.line();
            self.next();
            let right = self.parse_additive(sl)?;
            left = Expr::Bin(op, Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn parse_additive(&mut self, sl: bool) -> PResult<Expr> {
        let mut left = self.parse_multiplicative(sl)?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            let line = self.line();
            self.next();
            let right = self.parse_multiplicative(sl)?;
            left = Expr::Bin(op, Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self, sl: bool) -> PResult<Expr> {
        let mut left = self.parse_unary(sl)?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                Tok::Percent => BinOp::Mod,
                _ => break,
            };
            let line = self.line();
            self.next();
            let right = self.parse_unary(sl)?;
            left = Expr::Bin(op, Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn parse_unary(&mut self, sl: bool) -> PResult<Expr> {
        let line = self.line();
        match self.peek() {
            Tok::Minus => {
                self.next();
                let e = self.parse_unary(sl)?;
                Ok(Expr::Unary(UnOp::Neg, Box::new(e), line))
            }
            Tok::Bang => {
                self.next();
                let e = self.parse_unary(sl)?;
                Ok(Expr::Unary(UnOp::Not, Box::new(e), line))
            }
            _ => self.parse_postfix(sl),
        }
    }

    fn parse_postfix(&mut self, sl: bool) -> PResult<Expr> {
        let mut e = self.parse_primary(sl)?;
        loop {
            if *self.peek() == Tok::Dot {
                let line = self.line();
                self.next();
                let name = self.ident("field or method name after .")?;
                if *self.peek() == Tok::LParen {
                    // module call: only supported on a bare identifier base
                    if let Expr::Var(module, _) = &e {
                        let args = self.parse_args()?;
                        e = Expr::ModCall(module.clone(), name, args, line);
                    } else {
                        return Err((
                            line,
                            "method calls are only supported on modules in BOLD v1".into(),
                        ));
                    }
                } else {
                    e = Expr::Field(Box::new(e), name, line);
                }
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_args(&mut self) -> PResult<Vec<Expr>> {
        self.expect(Tok::LParen, "(")?;
        let mut args = Vec::new();
        while *self.peek() != Tok::RParen {
            args.push(self.parse_expr()?);
            if *self.peek() == Tok::Comma {
                self.next();
            }
        }
        self.expect(Tok::RParen, ") to close arguments")?;
        Ok(args)
    }

    fn parse_primary(&mut self, sl: bool) -> PResult<Expr> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Int(v) => {
                self.next();
                Ok(Expr::Int(v, line))
            }
            Tok::Float(v) => {
                self.next();
                Ok(Expr::Float(v, line))
            }
            Tok::Str(s) => {
                self.next();
                Ok(Expr::Str(s, line))
            }
            Tok::True => {
                self.next();
                Ok(Expr::Bool(true, line))
            }
            Tok::False => {
                self.next();
                Ok(Expr::Bool(false, line))
            }
            Tok::LParen => {
                self.next();
                let e = self.parse_expr()?;
                self.expect(Tok::RParen, ") to close grouping")?;
                Ok(e)
            }
            Tok::Ident(name) => {
                self.next();
                if *self.peek() == Tok::LParen {
                    let args = self.parse_args()?;
                    return Ok(Expr::Call(name, args, line));
                }
                if sl && *self.peek() == Tok::LBrace && self.struct_names.contains(&name) {
                    self.next(); // {
                    let mut fields = Vec::new();
                    while *self.peek() != Tok::RBrace {
                        let fname = self.ident("field name in struct literal")?;
                        self.expect(Tok::Colon, ": after field name")?;
                        let val = self.parse_expr()?;
                        fields.push((fname, val));
                        if *self.peek() == Tok::Comma {
                            self.next();
                        }
                    }
                    self.expect(Tok::RBrace, "} to close struct literal")?;
                    return Ok(Expr::StructLit(name, fields, line));
                }
                Ok(Expr::Var(name, line))
            }
            other => Err((line, format!("unexpected token {:?} in expression", other))),
        }
    }
}
