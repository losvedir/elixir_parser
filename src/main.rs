#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub elixir); // synthesized by LALRPOP

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod ast;
mod lexer;

#[cfg(test)]
use std::fs;

fn main() {
    println!("Hello, world!");
    let lexer = lexer::Lexer::new("defmodule Foo do def bar do end end");
    match elixir::DefModuleParser::new().parse(lexer) {
        Ok(res) => {
            dbg!(res);
        }
        Err(_err) => {}
    }
}

#[test]
fn parse1() {
    let elixir1 = fs::read_to_string("elixir/simple.ex").expect("elixir/simple.ex");
    let lexer = lexer::Lexer::new(&elixir1);
    assert_eq!(
        elixir::DefModuleParser::new().parse(lexer),
        Ok(ast::Expr::DefModule {
            name: "Foo".to_string(),
            exprs: vec![ast::Expr::DefFunc {
                name: "bar".to_string(),
                exprs: vec![ast::Expr::True]
            }]
        })
    )
}
