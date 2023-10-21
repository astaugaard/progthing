use std::rc::Rc;

pub enum Value {
    Str(Box<String>),
    Array(Box<Vec<Rc<Value>>>),
    Number(u64),
    Float(f64),
    Boolean(bool),
}
