use std::rc::Rc;

use crate::values::*;

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    Constant(usize),
    CallBuiltin(usize),
    CallFunction(usize),
    Add,
    Sub,
    Mult,
    Mod,
    Div,
    Or,
    And,
    Xor,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    NE,
    If(usize, usize),
    Concat,
    Return,
    Exit,
}

pub struct VmState {
    stack: Vec<Value>,
    call_stack: Vec<(usize, usize)>,
    function: usize,
    pc: usize,
}
impl VmState {
    pub fn get_instr(&mut self, program: &[Vec<OpCode>]) -> OpCode {
        *program
            .get(self.function)
            .expect("invalid function reference")
            .get(self.pc)
            .expect("invalid location pointed to by program counter")
    }

    pub fn push_value(&mut self, id: Value) {
        self.stack.push(id);
    }

    pub fn pop_value(&mut self) -> Value {
        self.stack.pop().expect("failed to find argument")
    }
}

pub fn execute(
    program: &[Vec<OpCode>],
    constant_table: &[Value],
    builtins: &mut [BuiltinFunc],
    main: usize,
) {
    let initial_state = VmState {
        stack: Vec::new(),
        call_stack: Vec::new(),
        function: main,
        pc: 0,
    };
    execute_helper(program, constant_table, builtins, initial_state)
}

fn execute_helper(
    program: &[Vec<OpCode>],
    constant_table: &[Value],
    builtins: &mut [BuiltinFunc],
    mut vm_state: VmState,
) {
    let mut terminate = false;
    while !terminate {
        terminate = executeInstr(program, constant_table, builtins, &mut vm_state);
    }
}

fn opcode_2args(vm_state: &mut VmState, function: fn(Value, Value) -> Value) {
    let a1 = vm_state.pop_value();
    let a2 = vm_state.pop_value();
    vm_state.push_value(function(a1, a2));
}

fn executeInstr(
    program: &[Vec<OpCode>],
    constant_table: &[Value],
    builtins: &mut [BuiltinFunc],
    vm_state: &mut VmState,
) -> bool {
    match vm_state.get_instr(program) {
        OpCode::Exit => return true,
        OpCode::Constant(id) => vm_state.push_value(constant_table[id].clone()),
        OpCode::CallBuiltin(id) => (builtins[id].function)(vm_state),
        OpCode::CallFunction(id) => {
            vm_state.call_stack.push((vm_state.function, vm_state.pc));
            vm_state.function = id;
            vm_state.pc = 0;
            return false;
        }
        OpCode::Return => {
            let (ret, loc) = vm_state
                .call_stack
                .pop()
                .expect("should always have something on the call stack when returning");
            vm_state.function = ret;
            vm_state.pc = loc;
        }
        OpCode::Add => opcode_2args(vm_state, add_values),
        OpCode::Sub => opcode_2args(vm_state, sub_values),
        OpCode::Mult => opcode_2args(vm_state, mult_values),
        OpCode::Mod => opcode_2args(vm_state, modulo_values),
        OpCode::Div => opcode_2args(vm_state, div_values),
        OpCode::Or => opcode_2args(vm_state, or_values),
        OpCode::And => opcode_2args(vm_state, and_values),
        OpCode::Xor => opcode_2args(vm_state, xor_values),
        OpCode::Concat => opcode_2args(vm_state, concat_values),
        OpCode::Lt => opcode_2args(vm_state, lt_values),
        OpCode::Gt => opcode_2args(vm_state, gt_values),
        OpCode::Le => opcode_2args(vm_state, le_values),
        OpCode::Ge => opcode_2args(vm_state, ge_values),
        OpCode::Eq => opcode_2args(vm_state, eq_values),
        OpCode::NE => opcode_2args(vm_state, ne_values),
        OpCode::If(t, f) => {
            vm_state.pc = if vm_state.pop_value().expect_bool() {
                t
            } else {
                f
            };
            return false;
        }
    }
    vm_state.pc += 1;
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_test() {
        let mut output: Vec<Value> = Vec::new();
        {
            let print_builtin = BuiltinFunc {
                function: Box::new(|vm: &mut VmState| output.push(vm.pop_value())),
            };

            let mut builtins = [print_builtin];

            execute(
                &[vec![
                    OpCode::Constant(0),
                    OpCode::CallBuiltin(0),
                    OpCode::Exit,
                ]],
                &[Value::Str(Rc::new("hello world".to_string()))],
                &mut builtins,
                0,
            );
        }

        assert_eq!(output, vec![Value::Str(Rc::new("hello world".to_string()))]);
    }

    #[test]
    fn call_test() {
        let mut output: Vec<Value> = Vec::new();
        {
            let print_builtin = BuiltinFunc {
                function: Box::new(|vm: &mut VmState| output.push(vm.pop_value())),
            };

            let mut builtins = [print_builtin];

            execute(
                &[
                    vec![
                        OpCode::CallFunction(1),
                        OpCode::CallBuiltin(0),
                        OpCode::Exit,
                    ],
                    vec![OpCode::Constant(0), OpCode::Return],
                ],
                &[Value::Str(Rc::new("hello world".to_string()))],
                &mut builtins,
                0,
            );
        }

        assert_eq!(output, vec![Value::Str(Rc::new("hello world".to_string()))]);
    }
}
