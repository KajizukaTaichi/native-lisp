fn main() {
    let code = include_str!("../example.lisp");
    let output = Compiler::build(code);
    println!("{}", output.unwrap());
}

struct Compiler {
    variables: Vec<String>,
}

impl Compiler {
    fn build(code: &str) -> Option<String> {
        let expr = Expr::parse(code)?;
        let mut compiler = Compiler {
            variables: Vec::new(),
        };
        let code = expr.compile(&mut compiler)?;
        let top = "section .text\n\tglobal _start\n\n_start:\n";
        let exit = "\tmov rdi, rax\n\tmov rax, 0x2000001\n\tsyscall\n";
        Some(format!("{top}{code}\n{exit}"))
    }
}

enum Expr {
    List(Vec<Expr>),
    Atom(Atom),
}

impl Expr {
    fn parse(source: &str) -> Option<Self> {
        let source = source.trim();
        if let Some(Some(source)) = source.strip_prefix("(").map(|x| x.strip_suffix(")")) {
            Some(Expr::List(
                tokenize(source)?
                    .iter()
                    .map(|x| Expr::parse(x))
                    .collect::<Option<Vec<_>>>()?,
            ))
        } else {
            Some(Expr::Atom(Atom::parse(source)?))
        }
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        match self {
            Expr::List(expr) => match expr.first()? {
                Expr::Atom(Atom::Symbol(func_name)) => {
                    macro_rules! multi_args {
                        ($order: expr) => {{
                            let mut result = expr.get(1)?.compile(ctx)?;
                            for arg in expr.iter().skip(2) {
                                let code = &format!(
                                    "\tpush rax\n{}\tmov rdx, rax\n\tpop rax\n\t{} rax, rdx\n",
                                    arg.compile(ctx)?,
                                    $order
                                );
                                result.push_str(code);
                            }
                            Some(result)
                        }};
                    }
                    match func_name.as_str() {
                        "+" => multi_args!("add"),
                        "-" => multi_args!("sub"),
                        "*" => multi_args!("imul"),
                        "/" => multi_args!("idiv"),
                        _ => None,
                    }
                }
                _ => None,
            },
            Expr::Atom(atom) => atom.compile(ctx),
        }
    }
}

enum Atom {
    Symbol(String),
    Integer(i64),
}

impl Atom {
    fn parse(source: &str) -> Option<Self> {
        let source = source.trim();
        if let Ok(n) = source.parse::<i64>() {
            Some(Atom::Integer(n))
        } else {
            Some(Atom::Symbol(source.to_string()))
        }
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        match self {
            Atom::Integer(number) => Some(format!("\tmov rax, {number}\n")),
            Atom::Symbol(name) => Some(format!("\tmov rax, [{name}]\n")),
        }
    }
}

pub fn tokenize(input: &str) -> Option<Vec<String>> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current_token = String::new();
    let mut in_parentheses: usize = 0;
    let mut in_quote = false;
    let mut is_escape = false;

    for c in input.chars() {
        if is_escape {
            current_token.push(match c {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                _ => c,
            });
            is_escape = false;
        } else {
            match c {
                '(' | '{' | '[' if !in_quote => {
                    current_token.push(c);
                    in_parentheses += 1;
                }
                ')' | '}' | ']' if !in_quote => {
                    current_token.push(c);
                    in_parentheses.checked_sub(1).map(|x| in_parentheses = x);
                }
                '"' | '\'' | '`' => {
                    in_quote = !in_quote;
                    current_token.push(c);
                }
                '\\' if in_quote => {
                    current_token.push(c);
                    is_escape = true;
                }
                other => {
                    if other.is_whitespace() && !in_quote {
                        if in_parentheses != 0 {
                            current_token.push(c);
                        } else if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                    } else {
                        current_token.push(c);
                    }
                }
            }
        }
    }

    // Syntax error check
    if is_escape || in_quote || in_parentheses != 0 {
        return None;
    }
    if !current_token.is_empty() {
        tokens.push(current_token.clone());
        current_token.clear();
    }
    Some(tokens)
}
