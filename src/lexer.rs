use itertools::Itertools;
use num_bigint::BigInt;
use num_bigint::ToBigInt;
use num_traits::Num;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Int(BigInt),
    Star,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexicalError {
    VersionControlMarker,
}

pub struct Lexer<'input> {
    chars: itertools::MultiPeek<std::str::Chars<'input>>,
    line: u32,
    col: u32,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            chars: itertools::multipeek(input.chars()),
            line: 0,
            col: 0,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // VC Merge Conflict
            if self.col == 0
                && self.match_char('<')
                && self.match_char('<')
                && self.match_char('<')
                && self.match_char('<')
                && self.match_char('<')
                && self.match_char('<')
                && self.match_char('<')
            {
                self.consume(7);
                self.consume_to_eol();
                return Some(Err(LexicalError::VersionControlMarker));
            }
            self.chars.reset_peek();

            // Base integers

            if self.match_char('0')
                && self.match_char('x')
                && self.match_fn(&|c| c.map_or(false, |ch| ch.is_digit(16)))
            {
                self.consume(2);

                let digits: String = self.chars.take_while_ref(|c| c.is_digit(16)).collect();
                self.col += digits.len() as u32;
                let val: BigInt = BigInt::from_str_radix(&digits, 16).unwrap();
                return Some(Ok((1, Tok::Int(val), 1)));
            }
            self.chars.reset_peek();

            if self.match_char('0')
                && self.match_char('b')
                && self.match_fn(&|c| c.map_or(false, |ch| ch.is_digit(2)))
            {
                self.consume(2);
                let digits: String = self.chars.take_while_ref(|c| c.is_digit(2)).collect();
                self.col += digits.len() as u32;
                let val: BigInt = BigInt::from_str_radix(&digits, 2).unwrap();
                return Some(Ok((1, Tok::Int(val), 1)));
            }
            self.chars.reset_peek();

            if self.match_char('0')
                && self.match_char('o')
                && self.match_fn(&|c| c.map_or(false, |ch| ch.is_digit(8)))
            {
                self.consume(2);
                let digits: String = self.chars.take_while_ref(|c| c.is_digit(8)).collect();
                self.col += digits.len() as u32;
                let val: BigInt = BigInt::from_str_radix(&digits, 8).unwrap();
                return Some(Ok((1, Tok::Int(val), 1)));
            }
            self.chars.reset_peek();

            // comments

            if self.match_char('#') {
                self.consume(1);
                self.consume_to_eol();
                continue;
            }
            self.chars.reset_peek();

            // flag for testing
            if self.match_char('*') {
                self.consume(1);
                return Some(Ok((1, Tok::Star, 1)));
            }
            self.chars.reset_peek();

            return None;
        }
    }
}

impl<'input> Lexer<'input> {
    fn consume(&mut self, n: u32) {
        for _ in 0..n {
            self.chars.next();
        }
        self.col += n;
        self.chars.reset_peek();
    }

    fn consume_to_eol(&mut self) {
        loop {
            match &self.chars.next() {
                Some('\n') => {
                    self.col = 0;
                    self.line += 1;
                    break;
                }
                None => {
                    self.col += 1;
                    break;
                }
                _ => {
                    self.col += 1;
                    continue;
                }
            }
        }
    }

    fn match_char(&mut self, c: char) -> bool {
        self.chars.peek() == Some(&c)
    }

    fn match_fn(&mut self, f: &Fn(Option<&char>) -> bool) -> bool {
        f(self.chars.peek())
    }
}

#[test]
fn lex1() {
    let mut lexer = Lexer::new("<<<<<<< VC conflict\n*");
    assert!(lexer.next() == Some(Err(LexicalError::VersionControlMarker)));
    assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
    assert!(lexer.next() == None);
}

#[test]
fn lex2() {
    let mut lexer = Lexer::new("0xf1A*0b110*0o73*");
    assert!(lexer.next() == Some(Ok((1, Tok::Int(0xf1a.to_bigint().unwrap()), 1))));
    assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
    assert!(lexer.next() == Some(Ok((1, Tok::Int(0b110.to_bigint().unwrap()), 1))));
    assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
    assert!(lexer.next() == Some(Ok((1, Tok::Int(0o73.to_bigint().unwrap()), 1))));
    assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
}

#[test]
fn lex3() {
    let mut lexer = Lexer::new("# this is a comment\n*");
    assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
}

#[test]
fn consume() {
    let mut lexer = Lexer::new("123456");
    assert!(lexer.chars.peek() == Some(&'1'));
    lexer.chars.reset_peek();
    lexer.consume(2);
    assert!(lexer.chars.peek() == Some(&'3'));
    lexer.consume(2);
    assert!(lexer.chars.peek() == Some(&'5'));
}

#[test]
fn match_char() {
    let mut lexer = Lexer::new("abc");
    assert!(lexer.match_char('a'));
    assert!(lexer.match_char('b'));
}

#[test]
fn match_fn() {
    let mut lexer = Lexer::new("abc");
    assert!(lexer.match_fn(&|ch| ch == Some(&'a')));
}
