use std::{arch::x86_64::_mm256_movemask_pd, collections::HashMap};

use crate::{
    bytecodevm::VmState,
    values::{BuiltinFunc, Value},
};

pub enum BuiltinValue<'a> {
    Constant(Value),
    Builtin(BuiltinFunc<'a>),
}

pub enum BuiltinId {
    Constant(usize),
    Builtin(usize),
}

pub struct BuiltinMaps<'a> {
    name_id_map: HashMap<&'a str, BuiltinId>,
    builtin_table: Vec<BuiltinFunc<'a>>,
    constant_table: Vec<Value>,
    builtin_name_table: Vec<&'a str>,
    constant_name_table: Vec<&'a str>,
}

fn generate_builtin_map<'a>(values: Vec<(&'a str, BuiltinValue<'a>)>) -> BuiltinMaps<'a> {
    let mut builtin_table: Vec<BuiltinFunc> = Vec::new();
    let mut constant_table: Vec<Value> = Vec::new();
    let mut builtin_name_table: Vec<&str> = Vec::new();
    let mut constant_name_table: Vec<&str> = Vec::new();
    let mut name_id_map: HashMap<&str, BuiltinId> = HashMap::new();
    for (k, v) in values.into_iter() {
        match v {
            BuiltinValue::Constant(val) => {
                name_id_map.insert(k, BuiltinId::Constant(constant_table.len()));
                constant_table.push(val);
                constant_name_table.push(k)
            }
            BuiltinValue::Builtin(func) => {
                name_id_map.insert(k.clone(), BuiltinId::Builtin(builtin_table.len()));
                builtin_table.push(func);
                builtin_name_table.push(k.clone())
            }
        }
    }
    BuiltinMaps {
        name_id_map,
        builtin_table,
        constant_table,
        builtin_name_table,
        constant_name_table,
    }
}

pub fn default_builtin_map() -> BuiltinMaps<'static> {
    generate_builtin_map(vec![(
        "dbg",
        BuiltinValue::Builtin(BuiltinFunc {
            function: Box::new(|vmstate: &mut VmState| print!("{:#?}", vmstate.pop_value())),
        }),
    )])
}


