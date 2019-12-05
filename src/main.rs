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

#[test]
fn parse_mix() {
    let mix = fs::read_to_string("elixir/mix.exs").expect("elixir/mix.exs");
    let lexer = lexer::Lexer::new(&mix);
    assert_eq!(
        elixir::DefModuleParser::new().parse(lexer).unwrap(),
        ast::Expr::DefModule {
            name: "TestApp.MixProject".to_string(),
            exprs: vec![
                ast::Expr::Use {
                    name: "Mix.Project".to_string()
                },
                ast::Expr::DefFunc {
                    name: "project".to_string(),
                    exprs: vec![ast::Expr::List {
                        elems: vec![
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "app".to_string()
                                    },
                                    ast::Expr::Atom {
                                        value: "test_app".to_string()
                                    }
                                ]
                            },
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "version".to_string()
                                    },
                                    ast::Expr::StringLit {
                                        value: "0.1.0".to_string()
                                    }
                                ]
                            },
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "elixir".to_string()
                                    },
                                    ast::Expr::StringLit {
                                        value: "~> 1.9".to_string()
                                    }
                                ]
                            },
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "start_permanent".to_string()
                                    },
                                    ast::Expr::FuncCall {
                                        name: "==".to_string(),
                                        args: vec![
                                            ast::Expr::FuncCall {
                                                name: "Mix.env".to_string(),
                                                args: vec![]
                                            },
                                            ast::Expr::Atom {
                                                value: "prod".to_string()
                                            }
                                        ]
                                    }
                                ]
                            },
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "deps".to_string()
                                    },
                                    ast::Expr::FuncCall {
                                        name: "deps".to_string(),
                                        args: vec![]
                                    }
                                ]
                            }
                        ]
                    }]
                },
                ast::Expr::DefFunc {
                    name: "application".to_string(),
                    exprs: vec![ast::Expr::List {
                        elems: vec![
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "extra_applications".to_string()
                                    },
                                    ast::Expr::List {
                                        elems: vec![ast::Expr::Atom {
                                            value: "logger".to_string()
                                        }]
                                    }
                                ]
                            },
                            ast::Expr::Tuple {
                                elems: vec![
                                    ast::Expr::Atom {
                                        value: "mod".to_string()
                                    },
                                    ast::Expr::Tuple {
                                        elems: vec![
                                            ast::Expr::Tbd {
                                                name: "TestApp.Application".to_string()
                                            },
                                            ast::Expr::List { elems: vec![] }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }]
                },
                ast::Expr::DefFunc {
                    name: "deps".to_string(),
                    exprs: vec![]
                }
            ]
        }
    )
}
