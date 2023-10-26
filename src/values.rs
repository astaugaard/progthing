use crate::bytecodevm::VmState;
use std::{
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub},
    rc::Rc,
};

pub struct BuiltinFunc<'a> {
    pub function: Box<dyn FnMut(&mut VmState) + 'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClosureFunc {
    Builtin(usize),
    Normal(usize, usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    total_args: u16,
    current_args: Vec<Rc<Value>>, // should be allocated to just enough args when it is intialized
    function: ClosureFunc,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(Rc<String>),
    Array(Rc<Vec<Value>>),
    Closure(Rc<Closure>),
    Number(u64),
    Float(f64),
    Boolean(bool),
}

impl Value {
    pub fn expect_bool(&self) -> bool {
        match self {
            Value::Boolean(a) => *a,
            _ => panic!("expected boolean")
        }
    }
}

// like call2_value but without the v1 and v2 so it makes a function that takes v1 and v2
macro_rules! value2_function {
    ($($ty:ident,$op:expr),*) => {
        |v1, v2| call2_value!(v1,v2,$($ty,$op),*)
    };
}


// call_value takes to args v1 and v2 and enumerates over the list of value types passed in
// checks if both of the args are the same type and if so calls the following op on them
//
// for example
// call2_value(v1,v2,Number,Add::add,Float,Mul::mul) would multiply v1 and v2 if they were floats
// add them if they are integers
// and error out otherwise
macro_rules! call2_value {
    ($v1:expr,$v2:expr,$($ty:ident,$op:expr),*) => {
        match ($v1, $v2) {
            $((Value::$ty(a),Value::$ty(b)) => Value::$ty($op(a,b)),)*
            _ => panic!("invalid types"),
        }
    };
}

// convience macro for numeric types to make defining the operations for them easier
macro_rules! call2_value_numeric {
    ($v1:expr,$v2:expr,$op:expr) => {
        call2_value!($v1, $v2, Number, $op, Float, $op)
    };
}

macro_rules! call2_delc {
    ($name:ident,$($ty:ident,$op:expr),*) => {
        pub fn $name(v1: Value,v2: Value) -> Value {
            call2_value!(v1,v2,$($ty,$op),*)
        }
    }
}

macro_rules! numeric2_delc {
    ($name:ident,$op:expr) => {
        call2_delc!($name, Number, $op, Float, $op);
    };
}

macro_rules! comparison_delc {
    ($name:ident,$op:expr,$op2:expr) => {
        pub fn $name(v1: Value, v2: Value) -> Value {
            match (v1, v2) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean($op(&a, &b)),
                (Value::Float(a), Value::Float(b)) => Value::Boolean($op2(&a, &b)),
                _ => panic!("invalid types"),
            }
        }
    };
}

fn add_rc_strings(mut s1: Rc<String>, s2: Rc<String>) -> Rc<String> {
    Rc::make_mut(&mut s1).push_str(&s2);
    s1
}

fn add_rc_vecs<A>(mut s1: Rc<Vec<A>>, s2: Rc<Vec<A>>) -> Rc<Vec<A>>
where
    A: Clone,
{
    Rc::make_mut(&mut s1).extend_from_slice(&s2);
    s1
}

numeric2_delc!(add_values, Add::add);
numeric2_delc!(sub_values, Sub::sub);
numeric2_delc!(mult_values, Mul::mul);
numeric2_delc!(div_values, Div::div);
call2_delc!(modulo_values, Number, Rem::rem);
call2_delc!(or_values, Number, BitOr::bitor);
call2_delc!(and_values, Number, BitAnd::bitand);
call2_delc!(xor_values, Number, BitXor::bitxor);
call2_delc!(concat_values, Str, add_rc_strings, Array, add_rc_vecs);

comparison_delc!(lt_values, PartialOrd::lt, PartialOrd::lt);
comparison_delc!(gt_values, PartialOrd::gt, PartialOrd::gt);
comparison_delc!(le_values, PartialOrd::le, PartialOrd::le);
comparison_delc!(ge_values, PartialOrd::ge, PartialOrd::ge);
comparison_delc!(eq_values, PartialEq::eq, PartialEq::eq);
comparison_delc!(ne_values, PartialEq::ne, PartialEq::ne);


