use crate::*;

const ARGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Expr {
    pub fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        match self {
            Expr::List(expr) => {
                macro_rules! pass_args {
                    () => {{
                        let mut args = String::new();
                        for (id, arg) in expr.iter().skip(1).enumerate() {
                            let code = arg.compile(ctx)?;
                            let code = &format!("{code}\tmov {}, rax\n", ARGS[id]);
                            args.push_str(code);
                        }
                        args
                    }};
                }
                macro_rules! stackframe {
                    ($order: expr) => {
                        format!(
                            "\n\tmov r10, [rel _ptr]\t; Stack frame\n\t{} r10, 512\n\tmov [rel _ptr], r10\n\n",
                            $order
                        )
                    };
                }
                macro_rules! call_func {
                    ($func_obj: expr) => {
                        Some(format!(
                            "{}{}\tmov r11, rax\n{}\tcall r11\n{}",
                            pass_args!(),
                            $func_obj.compile(ctx)?,
                            stackframe!("add"),
                            stackframe!("sub"),
                        ))
                    };
                }
                match expr.first()? {
                    Expr::Atom(Atom::Symbol(func_name)) => {
                        macro_rules! multi_args {
                            ($order: expr) => {{
                                let mut result = expr.get(1)?.compile(ctx)?;
                                for arg in expr.iter().skip(2) {
                                    let code = &format!(
                                        "\tpush rax\n{}\tmov rdx, rax\n\tpop rax\n\t{}\n",
                                        arg.compile(ctx)?,
                                        $order
                                    );
                                    result.push_str(code);
                                }
                                Some(result)
                            }};
                        }
                        macro_rules! declare_var {
                            ($name: expr) => {{
                                let addr = ctx.heap_addr;
                                ctx.variables.insert($name.to_string(), addr);
                                ctx.heap_addr += 8;
                                addr
                            }};
                        }
                        match func_name.strip_prefix("_")? {
                            "+" => multi_args!("add rax, rdx"),
                            "-" => multi_args!("sub rax, rdx"),
                            "*" => multi_args!("imul rax, rdx"),
                            "/" => multi_args!("mov rbx, rdx\n\txor rdx, rdx\n\tdiv rbx"),
                            "%" => multi_args!(
                                "mov rbx, rdx\n\txor rdx, rdx\n\tdiv rbx\n\tmov rax, rdx"
                            ),
                            "var" => {
                                let Expr::Atom(Atom::Symbol(name)) = expr.get(1)? else {
                                    return None;
                                };
                                let addr = declare_var!(name);
                                let value = expr.get(2)?.compile(ctx)?;
                                Some(format!("{value}\tmov [rel _ptr + {addr}], rax\n"))
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
                                    let addr = declare_var!(name);
                                    args.push(addr);
                                }
                                let receiver = args
                                    .iter()
                                    .enumerate()
                                    .map(|(id, addr)| {
                                        format!("\tmov [rel _ptr + {addr}], {}\n", ARGS[id])
                                    })
                                    .collect::<String>();
                                let body = &expr.get(2)?.compile(ctx)?;
                                let name = format!("lambda_{}", ctx.lambda_id);
                                ctx.lambda_id += 1;
                                ctx.functions
                                    .push(format!("{name}:\n{receiver}\n{body}\tret\n\n"));
                                Some(format!("\tlea rax, [rel {name}]\n"))
                            }
                            _ => call_func!(Expr::Atom(Atom::Symbol(func_name.to_owned()))),
                        }
                    }
                    func_obj => call_func!(func_obj),
                }
            }
            Expr::Atom(atom) => atom.compile(ctx),
        }
    }
}

impl Atom {
    pub fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        match self {
            Atom::Integer(number) => Some(format!("\tmov rax, {number}\n")),
            Atom::Symbol(name) => Some(format!(
                "\tmov rax, [rel _ptr + {}]\t; Load `{name}`\n",
                ctx.variables[name]
            )),
        }
    }
}
