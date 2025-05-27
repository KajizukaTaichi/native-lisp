use crate::*;

impl Expr {
    pub fn compile(&self, ctx: &mut Compiler) -> Option<String> {
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
                        "var" => {
                            let Expr::Atom(Atom::Symbol(name)) = expr.get(1)? else {
                                return None;
                            };
                            let value = expr.get(2)?.compile(ctx)?;
                            ctx.variables.insert(format!("\t{name} dq 0\n"));
                            Some(format!("{value}\tmov [rel {name}], rax\n"))
                        }
                        _ => None,
                    }
                }
                _ => None,
            },
            Expr::Atom(atom) => atom.compile(ctx),
        }
    }
}

impl Atom {
    pub fn compile(&self, _ctx: &mut Compiler) -> Option<String> {
        match self {
            Atom::Integer(number) => Some(format!("\tmov rax, {number}\n")),
            Atom::Symbol(name) => Some(format!("\tmov rax, [rel {name}]\n")),
        }
    }
}
