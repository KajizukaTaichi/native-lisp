mod codegen;
mod lexer;
mod parse;
use lexer::tokenize;
use std::collections::HashSet;

fn main() {
    let code = include_str!("../example.lisp");
    let output = Compiler::build(code);
    println!("{}", output.unwrap());
}

struct Compiler {
    functions: Vec<String>,
    variables: HashSet<String>,
}

impl Compiler {
    fn build(code: &str) -> Option<String> {
        let expr = tokenize(code)?
            .iter()
            .map(|code| Expr::parse(code))
            .collect::<Option<Vec<_>>>()?;
        let mut compiler = Compiler {
            functions: Vec::new(),
            variables: HashSet::new(),
        };
        let code = expr
            .iter()
            .map(|x| x.compile(&mut compiler))
            .collect::<Option<Vec<_>>>()?
            .concat();
        let top = "section .text\n\tglobal _start\n\n_start:\n";
        let exit = "\tmov rdi, rax\n\tmov rax, 0x2000001\n\tsyscall\n";
        let vars = compiler.variables.into_iter().collect::<String>();
        let fnc = compiler.functions.into_iter().collect::<String>();
        Some(format!("{top}{code}\n{exit}\n{fnc}\nsection .data\n{vars}"))
    }
}

enum Expr {
    List(Vec<Expr>),
    Atom(Atom),
}

enum Atom {
    Symbol(String),
    Integer(i64),
}
