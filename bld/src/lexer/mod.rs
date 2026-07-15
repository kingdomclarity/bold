// BOLD lexer: turns .bold source text into a token stream.
// Phase 1 of the bld compiler pipeline.

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // keywords
    Fn,
    Let,
    Mut,
    Struct,
    Import,
    If,
    Else,
    For,
    In,
    While,
    Return,
    True,
    False,
    // literals and identifiers
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    // punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Dot,
    DotDot,
    Arrow,
    // operators
    Assign,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    AndAnd,
    OrOr,
    Bang,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

pub fn lex(src: &str) -> Result<Vec<Token>, (usize, String)> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;

    macro_rules! push {
        ($t:expr) => {
            toks.push(Token { tok: $t, line })
        };
    }

    while i < chars.len() {
        let c = chars[i];

        // whitespace
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // comments
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // strings
        if c == '"' {
            let start_line = line;
            let mut s = String::new();
            i += 1;
            loop {
                if i >= chars.len() {
                    return Err((start_line, "unterminated string literal".into()));
                }
                let ch = chars[i];
                if ch == '"' {
                    i += 1;
                    break;
                }
                if ch == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        other => {
                            return Err((line, format!("unknown escape sequence \\{}", other)))
                        }
                    }
                    i += 1;
                    continue;
                }
                if ch == '\n' {
                    line += 1;
                }
                s.push(ch);
                i += 1;
            }
            push!(Tok::Str(s));
            continue;
        }

        // numbers
        if c.is_ascii_digit() {
            let mut num = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num.push(chars[i]);
                i += 1;
            }
            // a '.' is part of the number only if followed by a digit (not '..')
            if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
                num.push('.');
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    num.push(chars[i]);
                    i += 1;
                }
                let v: f64 = num
                    .parse()
                    .map_err(|_| (line, format!("invalid float literal {}", num)))?;
                push!(Tok::Float(v));
            } else {
                let v: i64 = num
                    .parse()
                    .map_err(|_| (line, format!("invalid integer literal {}", num)))?;
                push!(Tok::Int(v));
            }
            continue;
        }

        // identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            let mut id = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                id.push(chars[i]);
                i += 1;
            }
            let tok = match id.as_str() {
                "fn" => Tok::Fn,
                "let" => Tok::Let,
                "mut" => Tok::Mut,
                "struct" => Tok::Struct,
                "import" => Tok::Import,
                "if" => Tok::If,
                "else" => Tok::Else,
                "for" => Tok::For,
                "in" => Tok::In,
                "while" => Tok::While,
                "return" => Tok::Return,
                "true" => Tok::True,
                "false" => Tok::False,
                _ => Tok::Ident(id),
            };
            push!(tok);
            continue;
        }

        // operators and punctuation
        let two: String = chars[i..chars.len().min(i + 2)].iter().collect();
        let matched2 = match two.as_str() {
            "->" => Some(Tok::Arrow),
            ".." => Some(Tok::DotDot),
            "==" => Some(Tok::Eq),
            "!=" => Some(Tok::Ne),
            "<=" => Some(Tok::Le),
            ">=" => Some(Tok::Ge),
            "&&" => Some(Tok::AndAnd),
            "||" => Some(Tok::OrOr),
            _ => None,
        };
        if let Some(t) = matched2 {
            push!(t);
            i += 2;
            continue;
        }

        let matched1 = match c {
            '(' => Some(Tok::LParen),
            ')' => Some(Tok::RParen),
            '{' => Some(Tok::LBrace),
            '}' => Some(Tok::RBrace),
            ',' => Some(Tok::Comma),
            ':' => Some(Tok::Colon),
            '.' => Some(Tok::Dot),
            '=' => Some(Tok::Assign),
            '<' => Some(Tok::Lt),
            '>' => Some(Tok::Gt),
            '+' => Some(Tok::Plus),
            '-' => Some(Tok::Minus),
            '*' => Some(Tok::Star),
            '/' => Some(Tok::Slash),
            '%' => Some(Tok::Percent),
            '!' => Some(Tok::Bang),
            _ => None,
        };
        match matched1 {
            Some(t) => {
                push!(t);
                i += 1;
            }
            None => return Err((line, format!("unexpected character '{}'", c))),
        }
    }

    toks.push(Token {
        tok: Tok::Eof,
        line,
    });
    Ok(toks)
}
