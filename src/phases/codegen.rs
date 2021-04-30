use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::Ast;
use crate::types::Type;

static LABEL: AtomicUsize = AtomicUsize::new(0);

fn make_label() -> String {
    let value = LABEL.fetch_add(1, Ordering::SeqCst);
    format!(".L{}", value)
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub locals: BTreeMap<String, isize>,
    pub next_local_offset: isize,
}

impl Environment {
    pub fn new(locals: BTreeMap<String, isize>, next_local_offset: isize) -> Self {
        Environment {
            locals,
            next_local_offset,
        }
    }
}

pub trait CodeGenerator {
    fn emit(&self, buffer: &mut String, env: &mut Environment);
}

#[derive(Debug)]
pub struct Arm32Generator {
    ast: Ast,
}

impl CodeGenerator for Arm32Generator {
    fn emit(&self, buffer: &mut String, env: &mut Environment) {
        self.emit_ast(&self.ast, buffer, env);
    }
}

impl Arm32Generator {
    pub fn new(ast: Ast) -> Arm32Generator {
        Arm32Generator { ast }
    }

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

    fn emit_ast(&self, ast: &Ast, buffer: &mut String, env: &mut Environment) {
        match ast {
            Ast::Block(statements) => {
                buffer.push('\n');
                for statement in statements {
                    self.emit_ast(statement, buffer, env);
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
            Ast::ArrayLiteral(elements) => {
                let length = elements.len();
                let size = 4 * (length + 1); // +1 because we will have length stored as
                                             // a first word in the memory chunk
                buffer.push('\n');
                buffer.push_str(&format!("    ldr r0, ={}\n", size));
                buffer.push_str("    bl malloc\n");
                buffer.push_str("    push {r4, ip}\n");
                buffer.push_str("    mov r4, r0\n");
                buffer.push_str(&format!("    ldr r0, ={}\n", length));
                buffer.push_str("    str r0, [r4]\n"); // store length of array on the first byte

                for (i, elem) in elements.iter().enumerate() {
                    self.emit_ast(elem, buffer, env);
                    buffer.push_str(&format!("    str r0, [r4, #{}]\n", 4 * (i + 1)));
                }
                buffer.push_str("    mov r0, r4\n");
                buffer.push_str("    pop {r4, ip}\n");
            }
            Ast::ArrayLookup(array, index) => {
                self.emit_ast(array, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(index, buffer, env);
                buffer.push_str("    pop {r1, ip}\n"); // r1 points to first element of array
                buffer.push_str("    ldr r2, [r1]\n");
                buffer.push_str("    cmp r0, r2\n");
                buffer.push_str("    movhs r0, #0\n");
                buffer.push_str("    addlo r1, r1, #4\n");
                buffer.push_str("    lsllo r0, r0, #2\n"); // multiply index by four
                buffer.push_str("    ldrlo r0, [r1, r0]\n");
            }
            Ast::ArrayLength(array) => {
                self.emit_ast(array, buffer, env);
                buffer.push_str("    ldr r0, [r0, #0]\n");
            }
            Ast::Not(expr) => {
                buffer.push('\n');
                self.emit_ast(expr, buffer, env);
                buffer.push_str("    cmp r0, #0\n");
                buffer.push_str("    moveq r0, #1\n");
                buffer.push_str("    movne r0, #0\n");
            }
            Ast::Addition(lhs, rhs) => {
                buffer.push('\n');
                self.emit_ast(lhs, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(rhs, buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    add r0, r0, r1\n");
            }
            Ast::Subtraction(lhs, rhs) => {
                buffer.push('\n');
                self.emit_ast(lhs, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(rhs, buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    sub r0, r1, r0\n");
            }
            Ast::Multiplication(lhs, rhs) => {
                buffer.push('\n');
                self.emit_ast(lhs, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(rhs, buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    mul r0, r0, r1\n");
            }
            Ast::Division(lhs, rhs) => {
                buffer.push('\n');
                self.emit_ast(lhs, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(rhs, buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    udiv r0, r0, r1\n");
            }
            Ast::Equal(lhs, rhs) => {
                buffer.push('\n');
                self.emit_ast(lhs, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(rhs, buffer, env);
                buffer.push_str("    pop {r1, ip}\n");
                buffer.push_str("    cmp r0, r1\n");
                buffer.push_str("    moveq r0, #1\n");
                buffer.push_str("    movne r0, #0\n");
            }
            Ast::NotEqual(lhs, rhs) => {
                buffer.push('\n');
                self.emit_ast(lhs, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                self.emit_ast(rhs, buffer, env);
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
                        self.emit_ast(&args[0], buffer, env);
                        buffer.push_str(&format!("    bl {}\n", name));
                    }
                    x if x < 5 => {
                        buffer.push_str("    sub sp, sp, #16\n");
                        for (i, arg) in args.iter().enumerate() {
                            self.emit_ast(arg, buffer, env);
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
                self.emit_ast(expr, buffer, env);
                buffer.push_str("    push {r0, ip}\n");
                env.locals.insert(name.clone(), env.next_local_offset - 4);
                env.next_local_offset -= 8;
            }
            Ast::Assignment(name, expr) => {
                let offset = *env.locals.get(name).unwrap_or_else(|| {
                    panic!("Assignment to an undefined variable `{}`", name);
                });
                self.emit_ast(expr, buffer, env);
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
            Ast::Function(name, function_type, body) => {
                let (parameter_types, _return_type) = match function_type {
                    Type::Function {
                        parameter_types,
                        return_type,
                    } => (parameter_types, return_type),
                    _ => unreachable!(),
                };

                if parameter_types.len() > 4 {
                    panic!("More than four arguments are not supported");
                }

                buffer.push('\n');

                buffer.push('\n');
                buffer.push_str(&format!(".global {}\n", name));
                buffer.push_str(&format!("{}:\n", name));
                Arm32Generator::emit_prologue(buffer);
                let mut env = Arm32Generator::make_initial_function_environment(
                    parameter_types
                        .iter()
                        .map(|(x, _)| x)
                        .cloned()
                        .collect::<Vec<String>>()
                        .as_ref(),
                );
                self.emit_ast(body, buffer, &mut env);
                Arm32Generator::emit_epilogue(buffer);
            }
            Ast::Return(expr) => {
                self.emit_ast(expr, buffer, env);
                buffer.push_str("    mov sp, fp\n");
                buffer.push_str("    pop {fp, pc}\n");
            }
            Ast::If(condition, consequence, alternative) => {
                let false_label = make_label();
                let end_if_label = make_label();
                buffer.push('\n');
                self.emit_ast(condition, buffer, env);
                buffer.push_str("    cmp r0, #0\n");
                buffer.push_str(&format!("    beq {}\n", false_label));

                self.emit_ast(consequence, buffer, env);
                buffer.push_str(&format!("    b {}\n", end_if_label));

                buffer.push('\n');
                buffer.push_str(&format!("{}:", false_label));
                self.emit_ast(alternative, buffer, env);

                buffer.push('\n');
                buffer.push_str(&format!("{}:", end_if_label));
            }
            Ast::While(condition, block) => {
                let start_label = make_label();
                let end_label = make_label();
                buffer.push('\n');
                buffer.push_str(&format!("{}:\n", start_label));
                self.emit_ast(condition, buffer, env);
                buffer.push_str("    cmp r0, #0\n");
                buffer.push_str(&format!("    beq {}\n", end_label));

                self.emit_ast(block, buffer, env);
                buffer.push_str(&format!("    b {}\n", start_label));

                buffer.push_str(&format!("{}:", end_label));
            }
        }
    }
}
