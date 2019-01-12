pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Int,
    Star
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexicalError {
    VersionControlMarker
}

pub struct Lexer<'input> {
    chars: itertools::MultiPeek<std::str::Chars<'input>>,
    line: u32,
    col: u32,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer { chars: itertools::multipeek(input.chars()), line: 0, col: 0 }
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
                            Some('\n') => {
                                self.col = 0;
                                self.line += 1;
                                break
                            }
                            Some('\r') => {
                                self.col = 0;
                                self.line += 1;
                                break
                            }
                            None => {
                                self.col += 1;
                                break
                            }
                            _ => {
                                self.col += 1;
                                continue
                            }
                        }
                    }
                    return Some(Err(LexicalError::VersionControlMarker));
                }
            }

            if self.match1('*') {
                return Some(Ok((1, Tok::Star, 1)));
            }

            return None;
        }
    }
}

impl<'input> Lexer<'input> {
    fn match1(&mut self, c1: char) -> bool {
        if &self.chars.peek() == &Some(&c1) {
            &self.chars.next();
            self.col += 1;
            true
        } else {
            false
        }
    }

    fn match7(&mut self, c1: char, c2: char, c3: char, c4: char, c5: char, c6: char, c7: char) -> bool {
        if &self.chars.peek() == &Some(&c1) &&
                &self.chars.peek() == &Some(&c2) &&
                &self.chars.peek() == &Some(&c3) &&
                &self.chars.peek() == &Some(&c4) &&
                &self.chars.peek() == &Some(&c5) &&
                &self.chars.peek() == &Some(&c6) &&
                &self.chars.peek() == &Some(&c7) {
            for _ in 1..7 {
                &self.chars.next();
            }
            self.col += 7;
            true
        } else {
            &self.chars.reset_peek();
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