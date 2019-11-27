use std::str::Chars;
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Def,
    Defmodule,
    Defp,
    Do,
    End,
    ModName { name: String },
    FuncName { name: String },
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexicalError {}

#[derive(Debug)]
pub struct Lexer<'input> {
    chars: Chars<'input>,
    c0: Option<char>,
    c1: Option<char>,
    c2: Option<char>,
    c3: Option<char>,
    c4: Option<char>,
    c5: Option<char>,
    c6: Option<char>,
    c7: Option<char>,
    c8: Option<char>,
    position: usize,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.chars();
        let c0 = chars.next();
        let c1 = chars.next();
        let c2 = chars.next();
        let c3 = chars.next();
        let c4 = chars.next();
        let c5 = chars.next();
        let c6 = chars.next();
        let c7 = chars.next();
        let c8 = chars.next();

        Lexer {
            chars: chars,
            c0: c0,
            c1: c1,
            c2: c2,
            c3: c3,
            c4: c4,
            c5: c5,
            c6: c6,
            c7: c7,
            c8: c8,
            position: 0,
        }
    }
}

impl<'input> Lexer<'input> {
    fn shift(&mut self, n: usize) -> (usize, usize) {
        let start = self.position;
        for _ in 0..n {
            self.c0 = self.c1;
            self.c1 = self.c2;
            self.c2 = self.c3;
            self.c3 = self.c4;
            self.c4 = self.c5;
            self.c5 = self.c6;
            self.c6 = self.c7;
            self.c7 = self.c8;
            self.c8 = self.chars.next();
            self.position += 1;
        }
        let end = self.position;
        return (start, end);
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if (
                self.c0, self.c1, self.c2, self.c3, self.c4, self.c5, self.c6, self.c7, self.c8,
            ) == (
                Some('d'),
                Some('e'),
                Some('f'),
                Some('m'),
                Some('o'),
                Some('d'),
                Some('u'),
                Some('l'),
                Some('e'),
            ) {
                let (start, end) = self.shift(9);
                return Some(Ok((start, Tok::Defmodule, end)));
            }

            if (self.c0, self.c1, self.c2, self.c3) == (Some('d'), Some('e'), Some('f'), Some('p'))
            {
                let (start, end) = self.shift(4);
                return Some(Ok((start, Tok::Defp, end)));
            }

            if (self.c0, self.c1, self.c2) == (Some('d'), Some('e'), Some('f')) {
                let (start, end) = self.shift(3);
                return Some(Ok((start, Tok::Def, end)));
            }

            if (self.c0, self.c1, self.c2) == (Some('e'), Some('n'), Some('d')) {
                let (start, end) = self.shift(3);
                return Some(Ok((start, Tok::End, end)));
            }

            if (self.c0, self.c1) == (Some('d'), Some('o')) {
                let (start, end) = self.shift(2);
                return Some(Ok((start, Tok::Do, end)));
            }

            if self.c0.map_or(false, |c| c.is_uppercase()) {
                let mut name: Vec<char> = vec![];
                let start = self.position;
                while self.c0.map_or(false, |c| is_valid_module_name_char(&c)) {
                    name.push(self.c0.unwrap());
                    self.shift(1);
                }
                let end = self.position;
                return Some(Ok((
                    start,
                    Tok::ModName {
                        name: name.iter().collect(),
                    },
                    end,
                )));
            }

            if self.c0.map_or(false, |c| c.is_lowercase()) {
                let mut name: Vec<char> = vec![];
                let start = self.position;
                while self.c0.map_or(false, |c| is_valid_function_name_char(&c)) {
                    name.push(self.c0.unwrap());
                    self.shift(1);
                }
                let end = self.position;
                return Some(Ok((
                    start,
                    Tok::FuncName {
                        name: name.iter().collect(),
                    },
                    end,
                )));
            }

            if self.c0 == Some(' ') || self.c0 == Some('\n') {
                self.shift(1);
                continue;
            }

            if self.c0 == None {
                return None;
            }

            self.shift(1);
        }
    }
}

fn is_valid_module_name_char(c: &char) -> bool {
    c.is_alphanumeric() || c == &'.' || c == &'_'
}

fn is_valid_function_name_char(c: &char) -> bool {
    c.is_alphanumeric() || c == &'_' || c == &'?' || c == &'!'
}

#[test]
fn lex1() {
    let mut lexer = Lexer::new("defmodule Foo.Baz do\n  def bar? do\n  end\nend\n");
    assert_eq!(lexer.next(), Some(Ok((0, Tok::Defmodule, 9))));
    assert_eq!(
        lexer.next(),
        Some(Ok((
            10,
            Tok::ModName {
                name: "Foo.Baz".to_string()
            },
            17
        )))
    );
    assert_eq!(lexer.next(), Some(Ok((18, Tok::Do, 20))));
    assert_eq!(lexer.next(), Some(Ok((23, Tok::Def, 26))));
    assert_eq!(
        lexer.next(),
        Some(Ok((
            27,
            Tok::FuncName {
                name: "bar?".to_string()
            },
            31
        )))
    );
    assert_eq!(lexer.next(), Some(Ok((32, Tok::Do, 34))));
    assert_eq!(lexer.next(), Some(Ok((37, Tok::End, 40))));
    assert_eq!(lexer.next(), Some(Ok((41, Tok::End, 44))));
}

// #[test]
// fn lex1() {
//     let mut lexer = Lexer::new("<<<<<<< VC conflict\n*");
//     assert!(lexer.next() == Some(Err(LexicalError::VersionControlMarker)));
//     assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
//     assert!(lexer.next() == None);
// }

// #[test]
// fn lex2() {
//     let mut lexer = Lexer::new("0xf1A*0b110*0o73*");
//     assert!(lexer.next() == Some(Ok((1, Tok::Int(0xf1a.to_bigint().unwrap()), 1))));
//     assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
//     assert!(lexer.next() == Some(Ok((1, Tok::Int(0b110.to_bigint().unwrap()), 1))));
//     assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
//     assert!(lexer.next() == Some(Ok((1, Tok::Int(0o73.to_bigint().unwrap()), 1))));
//     assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
// }

// #[test]
// fn lex3() {
//     let mut lexer = Lexer::new("# this is a comment\n*");
//     assert!(lexer.next() == Some(Ok((1, Tok::Star, 1))));
// }

// #[test]
// fn consume() {
//     let mut lexer = Lexer::new("123456");
//     assert!(lexer.chars.peek() == Some(&'1'));
//     lexer.chars.reset_peek();
//     lexer.consume(2);
//     assert!(lexer.chars.peek() == Some(&'3'));
//     lexer.consume(2);
//     assert!(lexer.chars.peek() == Some(&'5'));
// }

// #[test]
// fn match_char() {
//     let mut lexer = Lexer::new("abc");
//     assert!(lexer.match_char('a'));
//     assert!(lexer.match_char('b'));
// }

// #[test]
// fn match_fn() {
//     let mut lexer = Lexer::new("abc");
//     assert!(lexer.match_fn(&|ch| ch == &'a'));
// }
