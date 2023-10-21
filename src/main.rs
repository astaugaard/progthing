use std::{io, fs};
use mylanguage::parser::parser;
use mylanguage::bytecodevm::*;
pub mod values;

// use crate::bytecodevm::*;
mod bytecodevm;

fn main() -> io::Result<()> {
    // let mut buffer = String::new();
    // let stdin = io::stdin();
    // stdin.read_line(&mut buffer)?;

    // buffer.truncate(buffer.len() - 1);
    let string:Box<String> = Box::new("".to_string());

    println!("boxed string size: {}", core::mem::size_of_val(&string));

    let input = fs::read_to_string("testfiles/testFunc.txt")?;

    let parserResult = parser().parse(input.as_bytes());

    print!("{:#?}",parserResult);


    Ok(())
}
