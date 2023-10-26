use std::collections::HashMap;

use crate::{builtins::BuiltinMaps, parser::Decleration, bytecodevm::OpCode, values::{Value, BuiltinFunc}};



fn compile<'a>(value_maps: &BuiltinMaps<'a>, delcs: Vec<Decleration>) -> (Vec<Vec<OpCode>>,Vec<Value>,usize) {
    let (res_delcs,global_var_map) = resolve_variables(delcs);
    let constant_map: HashMap<String,usize> = HashMap::new();
    let constant_vec: Vec<Value> = Vec::new();
    let comps = res_delcs.iter().map(|a| compile_delc(&mut constant_map, &mut constant_vec, a));

    (comps.collect(),constant_vec,global_var_map.get("main"))
}

fn resolve_variables(delcs: Vec<Decleration>) -> (_,HashMap<String,usize>) {
    todo!()
}

fn compile_delc(constant_map: &mut HashMap<String, usize>, constant_vec: &[Value], a: _) -> Vec<OpCode> {
    todo!()
}
