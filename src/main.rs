mod r#gen;
mod lexer;
mod parse;
use lexer::tokenize;

use indexmap::IndexMap;

fn main() {
    let code = include_str!("../example.lisp");
    let output = Compiler::build(code);
    println!("{}", output.unwrap());
}

struct Compiler {
    lambda_id: usize,
    heap_addr: usize,
    functions: Vec<String>,
    variables: IndexMap<String, usize>,
}

impl Compiler {
    fn build(code: &str) -> Option<String> {
        let expr = tokenize(code)?
            .iter()
            .map(|code| Expr::parse(code))
            .collect::<Option<Vec<_>>>()?;
        let mut compiler = Compiler {
            lambda_id: 0,
            heap_addr: 0,
            functions: Vec::new(),
            variables: IndexMap::new(),
        };
        let code = expr
            .iter()
            .map(|x| x.compile(&mut compiler))
            .collect::<Option<Vec<_>>>()?
            .concat();
        let bss = "section .bss\n\theap: resb 65536\n\tptr: resq 1\n";
        let top = "section .text\n\talign 16\n\tglobal _start\n\n_start:\n";
        let exit = "\tmov rdi, rax\n\tmov rax, 0x2000001\n\tsyscall\n\n";
        let fnc = compiler.functions.into_iter().collect::<String>();
        Some(format!("{bss}{top}mov [rel ptr], rax\n{code}{exit}{fnc}"))
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
