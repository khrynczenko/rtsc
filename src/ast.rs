use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static LABEL: AtomicUsize = AtomicUsize::new(0);

fn make_label() -> String {
    let value = LABEL.fetch_add(1, Ordering::SeqCst);
    format!(".L{}", value)
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    locals: HashMap<String, isize>,
    next_local_offset: isize,
}

impl Environment {
    pub fn new(locals: HashMap<String, isize>, next_local_offset: isize) -> Self {
        Environment {
            locals,
            next_local_offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Null,
    Undefined,
    Number(i32),
    Bool(bool),
    Identifier(String),
    Not(Box<Ast>),
    Equal(Box<Ast>, Box<Ast>),
    NotEqual(Box<Ast>, Box<Ast>),
    Addition(Box<Ast>, Box<Ast>),
    Subtraction(Box<Ast>, Box<Ast>),
    Multiplication(Box<Ast>, Box<Ast>),
    Division(Box<Ast>, Box<Ast>),
    Call(String, Vec<Ast>),
    Return(Box<Ast>),
    Block(Vec<Ast>),
    If(Box<Ast>, Box<Ast>, Box<Ast>),
    Function(String, Vec<String>, Box<Ast>),
    Var(String, Box<Ast>),
    Assignment(String, Box<Ast>),
    While(Box<Ast>, Box<Ast>),
}

impl Ast {
    fn make_initial_function_environment(params: &[String]) -> Environment {
        let locals = params
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), 4 * i as isize - 16))
            .collect();
        Environment::new(locals, -20)
    }
    fn emit_prologue(buffer: &mut String) {
        buffer.push_str("    push {fp, lr}\n");
        buffer.push_str("    mov fp, sp\n");
        buffer.push_str("    push {r0, r1, r2, r3}\n");
    }
    fn emit_epilogue(buffer: &mut String) {
        buffer.push_str("    mov sp, fp\n"); // deallocate stack
        buffer.push_str("    mov r0, #0\n");
        buffer.push_str("    pop {fp, pc}\n");
    }
    pub fn emit(&self, buffer: &mut String, env: &mut Environment) {
        match self {
            Ast::Block(statements) => {
                buffer.push('\n');
                for statement in statements {
                    statement.emit(buffer, env)
                }
            }
            Ast::Undefined => {
                buffer.push('\n');
                buffer.push_str("    mov r0, #0\n");
            }
            Ast::Null => {
                buffer.push('\n');
                buffer.push_str("    mov r0, #0\n");
            }
            Ast::Number(value) => {
                buffer.push('\n');
                buffer.push_str(&format!("    ldr r0, ={}\n", value));
            }
            Ast::Bool(value) => {
                buffer.push('\n');
                if *value {
                    buffer.push_str("    mov r0, #1\n");
                } else {
                    buffer.push_str("    mov r0, #0\n");
                }
            }
            Ast::Not(expr) => {
                buffer.push('\n');
                expr.emit(buffer, env);
                buffer.push_str("    cmp r0, #0\n");
                buffer.push_str("    moveq r0, #1\n");
                buffer.push_str("    movne r0, #0\n");
            }
            Ast::Addition(lhs, rhs) => {
                buffer.push('\n');
                lhs.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                rhs.emit(buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    add r0, r0, r1\n");
            }
            Ast::Subtraction(lhs, rhs) => {
                buffer.push('\n');
                lhs.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                rhs.emit(buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    sub r0, r1, r0\n");
            }
            Ast::Multiplication(lhs, rhs) => {
                buffer.push('\n');
                lhs.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                rhs.emit(buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    mul r0, r0, r1\n");
            }
            Ast::Division(lhs, rhs) => {
                buffer.push('\n');
                lhs.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                rhs.emit(buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    udiv r0, r0, r1\n");
            }
            Ast::Equal(lhs, rhs) => {
                buffer.push('\n');
                lhs.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                rhs.emit(buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    cmp r0, r1\n");
                buffer.push_str("    moveq r0, #1\n");
                buffer.push_str("    movne r0, #0\n");
            }
            Ast::NotEqual(lhs, rhs) => {
                buffer.push('\n');
                lhs.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                rhs.emit(buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    cmp r0, r1\n");
                buffer.push_str("    moveq r0, #0\n");
                buffer.push_str("    movne r0, #1\n");
            }
            Ast::Call(name, args) => {
                buffer.push('\n');
                match args.len() {
                    0 => {
                        buffer.push_str(&format!("    bl {}\n", name));
                    }
                    1 => {
                        args[0].emit(buffer, env);
                        buffer.push_str(&format!("    bl {}\n", name));
                    }
                    x if x < 5 => {
                        buffer.push_str("    sub sp, sp, #16\n");
                        for (i, arg) in args.iter().enumerate() {
                            arg.emit(buffer, env);
                            buffer.push_str(&format!("    str r0, [sp, #{}]\n", 4 * i));
                        }
                        buffer.push_str("    pop {r0, r1, r2, r3}\n");
                        buffer.push_str(&format!("    bl {}\n", name));
                    }
                    _ => {
                        panic!("More than four arguments are not supported");
                    }
                }
            }
            Ast::Var(name, expr) => {
                buffer.push('\n');
                expr.emit(buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                env.locals.insert(name.clone(), env.next_local_offset - 4);
                env.next_local_offset -= 8;
            }
            Ast::Assignment(name, expr) => {
                let offset = *env.locals.get(name).unwrap_or_else(|| {
                    panic!("Assignment to an undefined variable `{}`", name);
                });
                expr.emit(buffer, env);
                buffer.push_str(&format!("    str r0, [fp, #{}]\n", offset));
            }
            Ast::Identifier(name) => {
                buffer.push('\n');
                let offset = env.locals.get(name);
                if let Some(offset) = offset {
                    buffer.push_str(&format!("    ldr r0, [fp, #{}]\n", offset));
                } else {
                    panic!("Tried to use an undefined name {}", name);
                }
            }
            Ast::Function(name, parameters, body) => {
                buffer.push('\n');
                if parameters.len() > 4 {
                    panic!("More than four arguments are not supported");
                }

                buffer.push('\n');
                buffer.push_str(&format!(".global {}\n", name));
                buffer.push_str(&format!("{}:\n", name));
                Ast::emit_prologue(buffer);
                let mut env = Ast::make_initial_function_environment(parameters);
                body.emit(buffer, &mut env);
                Ast::emit_epilogue(buffer);
            }
            Ast::Return(expr) => {
                expr.emit(buffer, env);
                buffer.push_str("    mov sp, fp\n");
                buffer.push_str("    pop {fp, pc}\n");
            }
            Ast::If(condition, consequence, alternative) => {
                let false_label = make_label();
                let end_if_label = make_label();
                buffer.push('\n');
                condition.emit(buffer, env);
                buffer.push_str("    cmp r0, #0\n");
                buffer.push_str(&format!("    beq {}\n", false_label));

                consequence.emit(buffer, env);
                buffer.push_str(&format!("    b {}\n", end_if_label));

                buffer.push('\n');
                buffer.push_str(&format!("{}:", false_label));
                alternative.emit(buffer, env);

                buffer.push('\n');
                buffer.push_str(&format!("{}:", end_if_label));
            }
            Ast::While(condition, block) => {
                let start_label = make_label();
                let end_label = make_label();
                buffer.push('\n');
                buffer.push_str(&format!("{}:\n", start_label));
                condition.emit(buffer, env);
                buffer.push_str("    cmp r0, #0\n");
                buffer.push_str(&format!("    beq {}\n", end_label));

                block.emit(buffer, env);
                buffer.push_str(&format!("    b {}\n", start_label));

                buffer.push_str(&format!("{}:", end_label));
            }
        }
    }
}
