#[derive(Debug, PartialEq)]
pub enum Expr {
  DefFunc { name: String, exprs: Vec<Expr> },
  DefModule { name: String, exprs: Vec<Expr> },

  Use { name: String },

  List { elems: Vec<Expr> },
  Tuple { elems: Vec<Expr> },
  Atom { value: String },
  StringLit { value: String },

  FuncCall { name: String, args: Vec<Expr> },
  True,

  Tbd { name: String },
}
