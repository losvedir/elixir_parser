#[macro_use] extern crate lalrpop_util;
extern crate itertools;
lalrpop_mod!(pub elixir); // synthesized by LALRPOP

pub mod lexer;

// use std::fs;

fn main() {
    println!("Hello, world!");
}

#[test]
fn calculator1() {
    // let contents = fs::read_to_string("elixir/simple.ex")
    //     .expect("Something went wrong reading the file");
    // assert!(elixir::TermParser::new().parse(&contents).is_ok());
}