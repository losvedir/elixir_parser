#[derive(Debug, PartialEq)]
pub enum Expr {
  DefFunc { name: String, exprs: Vec<Expr> },

  DefModule { name: String, exprs: Vec<Expr> },

  Use { modname: String },

  True,
}
