mod r#gen;
mod lexer;
mod parse;
use lexer::tokenize;

fn main() {
    let code = include_str!("../example.lisp");
    let output = Compiler::build(code);
    println!("{}", output.unwrap());
}

struct Compiler {
    lambda_id: usize,
    functions: Vec<String>,
    variables: Vec<String>,
}

impl Compiler {
    fn build(code: &str) -> Option<String> {
        let expr = tokenize(code)?
            .iter()
            .map(|code| Expr::parse(code))
            .collect::<Option<Vec<_>>>()?;
        let mut compiler = Compiler {
            lambda_id: 0,
            functions: Vec::new(),
            variables: Vec::new(),
        };
        let code = expr
            .iter()
            .map(|x| x.compile(&mut compiler))
            .collect::<Option<Vec<_>>>()?
            .concat();
        let bss = "section .bss\n\theap_start:\tresb 65536\n\theap_ptr:\tresq 1\n";
        let top = "section .text\n\talign 16\n\tglobal _start\n\n_start:\n";
        let exit = "\tmov rdi, rax\n\tmov rax, 0x2000001\n\tsyscall\n";
        let vars = compiler.variables.into_iter().collect::<String>();
        let fnc = compiler.functions.into_iter().collect::<String>();
        let vars = format!("\nsection .data\n{vars}");
        Some(format!("{bss}{top}{code}{exit}{fnc}{vars}"))
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
