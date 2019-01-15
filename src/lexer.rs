pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Int,
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
            if self.col == 0 {
                if self.match7('<', '<', '<', '<', '<', '<', '<') {
                    loop {
                        match &self.chars.next() {
                            Some('\n') | Some('\r') => {
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
                    return Some(Err(LexicalError::VersionControlMarker));
                }
            }
            self.chars.reset_peek();

            // Base integers

            if self.match_char('0')
                && self.match_char('x')
                && self.match_fn(&is_hex)
            {
                return None;
            }
            self.chars.reset_peek();

            if self.match_char('*') {
                self.consume(1);
                return Some(Ok((1, Tok::Star, 1)));
            }
            self.chars.reset_peek();

            return None;
        }
    }
}

fn is_hex(c: Option<&char>) -> bool {
    match c {
        Some('0') | Some('1') | Some('2') | Some('3') | Some('4') | Some('5') | Some('6')
        | Some('7') | Some('8') | Some('9') | Some('a') | Some('A') | Some('b') | Some('B')
        | Some('c') | Some('C') | Some('d') | Some('D') | Some('e') | Some('E') | Some('f')
        | Some('F') => true,
        _ => false,
    }
}

impl<'input> Lexer<'input> {
    fn consume(&mut self, n: u32) {
        for _ in 0..n {
            self.chars.next();
        }
        self.col += n;
    }

    fn match_char(&mut self, c: char) -> bool {
        self.chars.peek() == Some(&c)
    }

    fn match_fn(&mut self, f: &Fn(Option<&char>) -> bool) -> bool {
        f(self.chars.peek())
    }

    fn match7(
        &mut self,
        c1: char,
        c2: char,
        c3: char,
        c4: char,
        c5: char,
        c6: char,
        c7: char,
    ) -> bool {
        if self.chars.peek() == Some(&c1)
            && self.chars.peek() == Some(&c2)
            && self.chars.peek() == Some(&c3)
            && self.chars.peek() == Some(&c4)
            && self.chars.peek() == Some(&c5)
            && self.chars.peek() == Some(&c6)
            && self.chars.peek() == Some(&c7)
        {
            for _ in 1..7 {
                self.chars.next();
            }
            self.col += 7;
            true
        } else {
            self.chars.reset_peek();
            false
        }
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