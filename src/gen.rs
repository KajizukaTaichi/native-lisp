use crate::*;

impl Expr {
    pub fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        match self {
            Expr::List(expr) => {
                macro_rules! pass_args {
                    () => {{
                        let mut args = String::new();
                        for (id, arg) in expr.iter().skip(1).enumerate() {
                            let code = arg.compile(ctx)?;
                            let code = &format!("{code}\tmov r{}, rax\n", id + 8);
                            args.push_str(code);
                        }
                        args
                    }};
                }
                match expr.first()? {
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
                        match func_name.replacen("lisp_", "", 1).as_str() {
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
                            "lambda" => {
                                let Expr::List(list) = expr.get(1)? else {
                                    return None;
                                };
                                let mut args = vec![];
                                for arg in list {
                                    let Expr::Atom(Atom::Symbol(name)) = arg else {
                                        return None;
                                    };
                                    ctx.variables.insert(format!("\t{name} dq 0\n"));
                                    args.push(name);
                                }
                                let name = format!("lambda_{}", ctx.lambda_id);
                                ctx.lambda_id += 1;
                                let receiver = args
                                    .iter()
                                    .enumerate()
                                    .map(|(id, name)| format!("\tmov [rel {name}], r{}\n", id + 8))
                                    .collect::<Vec<_>>()
                                    .concat();
                                let body = &expr.get(2)?.compile(ctx)?;
                                ctx.functions
                                    .push(format!("{name}:\n{receiver}\n{body}\tret\n\n"));
                                Some(format!("\tlea rax, [rel {name}]\n"))
                            }
                            _ => Some(format!(
                                "{}\tmov rax, [rel {func_name}]\n\tcall rax\n",
                                pass_args!()
                            )),
                        }
                    }
                    func_obj => {
                        let code = func_obj.compile(ctx)?;
                        Some(format!("{}{code}\tcall rax\n", pass_args!()))
                    }
                }
            }
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
