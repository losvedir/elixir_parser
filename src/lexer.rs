use std::str::Chars;
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Def,
    DefModule,
    Defp,
    Do,
    End,
    ModName { name: String },
    FuncName { name: String },
    True,
    Use,
    Lbrace,
    Rbrace,
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

    fn matches_literal(&mut self, lit: &str) -> Option<(usize, usize)> {
        let mut chars = lit.chars();

        let (matches, len) = match lit.chars().count() {
            1 => (chars.next() == self.c0, 1),
            2 => (chars.next() == self.c0 && chars.next() == self.c1, 2),
            3 => (
                chars.next() == self.c0 && chars.next() == self.c1 && chars.next() == self.c2,
                3,
            ),
            4 => (
                chars.next() == self.c0
                    && chars.next() == self.c1
                    && chars.next() == self.c2
                    && chars.next() == self.c3,
                4,
            ),
            5 => (
                chars.next() == self.c0
                    && chars.next() == self.c1
                    && chars.next() == self.c2
                    && chars.next() == self.c3
                    && chars.next() == self.c4,
                5,
            ),
            6 => (
                chars.next() == self.c0
                    && chars.next() == self.c1
                    && chars.next() == self.c2
                    && chars.next() == self.c3
                    && chars.next() == self.c4
                    && chars.next() == self.c5,
                6,
            ),
            7 => (
                chars.next() == self.c0
                    && chars.next() == self.c1
                    && chars.next() == self.c2
                    && chars.next() == self.c3
                    && chars.next() == self.c4
                    && chars.next() == self.c5
                    && chars.next() == self.c6,
                7,
            ),
            8 => (
                chars.next() == self.c0
                    && chars.next() == self.c1
                    && chars.next() == self.c2
                    && chars.next() == self.c3
                    && chars.next() == self.c4
                    && chars.next() == self.c5
                    && chars.next() == self.c6
                    && chars.next() == self.c7,
                8,
            ),
            9 => (
                chars.next() == self.c0
                    && chars.next() == self.c1
                    && chars.next() == self.c2
                    && chars.next() == self.c3
                    && chars.next() == self.c4
                    && chars.next() == self.c5
                    && chars.next() == self.c6
                    && chars.next() == self.c7
                    && chars.next() == self.c8,
                9,
            ),
            _ => unreachable!(),
        };
        if matches {
            let start = self.position;
            self.shift(len);
            let end = self.position;
            return Some((start, end));
        } else {
            return None;
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((start, end)) = self.matches_literal("[") {
                return Some(Ok((start, Tok::Lbrace, end)));
            }
            if let Some((start, end)) = self.matches_literal("]") {
                return Some(Ok((start, Tok::Rbrace, end)));
            }
            if let Some((start, end)) = self.matches_literal("defmodule") {
                return Some(Ok((start, Tok::DefModule, end)));
            }

            if let Some((start, end)) = self.matches_literal("defp") {
                return Some(Ok((start, Tok::Defp, end)));
            }

            if let Some((start, end)) = self.matches_literal("def") {
                return Some(Ok((start, Tok::Def, end)));
            }

            if let Some((start, end)) = self.matches_literal("end") {
                return Some(Ok((start, Tok::End, end)));
            }

            if let Some((start, end)) = self.matches_literal("do") {
                return Some(Ok((start, Tok::Do, end)));
            }

            if let Some((start, end)) = self.matches_literal("true") {
                return Some(Ok((start, Tok::True, end)));
            }

            if let Some((start, end)) = self.matches_literal("use") {
                return Some(Ok((start, Tok::Use, end)));
            }

            if let Some((_, _)) = self.matches_literal("#") {
                while self.c1 != Some('\n') && self.c1 != Some('\r') && self.c1 != None {
                    self.shift(1);
                }

                // CLRF (\r\n)
                if self.c1 == Some('\n') {
                    self.shift(1);
                }

                continue;
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

            if self.c0 == Some(' ') || self.c0 == Some('\n') || self.c0 == Some('\r') {
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
    assert_eq!(lexer.next(), Some(Ok((0, Tok::DefModule, 9))));
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

#[test]
fn lex_comments() {
    let mut lexer = Lexer::new("# this is a comment to end of line\ntrue");
    assert_matches!(lexer.next(), Some(Ok((_, Tok::True, _))));
}
