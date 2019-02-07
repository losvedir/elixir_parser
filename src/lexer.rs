use itertools::Itertools;
use num_bigint::BigInt;
use num_bigint::ToBigInt;
use num_traits::Num;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Atom(String),
    Char(char),
    Int(BigInt),
    KwIdentifier(String),
    Star,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexicalError {
    VersionControlMarker,
    InvalidSigil,
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
            // tokenize(("<<<<<<<" ++ _) = Original, Line, 1, _Scope, Tokens) ->
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
            // tokenize([$0, $x, H | T], Line, Column, Scope, Tokens) when ?is_hex(H) ->
            if self.match_char('0') && self.match_char('x') && self.match_fn(&|ch| ch.is_digit(16))
            {
                self.consume(2);

                let digits: String = self.chars.take_while_ref(|c| c.is_digit(16)).collect();
                self.col += digits.len() as u32;
                let val: BigInt = BigInt::from_str_radix(&digits, 16).unwrap();
                return Some(Ok((1, Tok::Int(val), 1)));
            }
            self.chars.reset_peek();

            // tokenize([$0, $b, H | T], Line, Column, Scope, Tokens) when ?is_bin(H) ->
            if self.match_char('0') && self.match_char('b') && self.match_fn(&|ch| ch.is_digit(2)) {
                self.consume(2);
                let digits: String = self.chars.take_while_ref(|c| c.is_digit(2)).collect();
                self.col += digits.len() as u32;
                let val: BigInt = BigInt::from_str_radix(&digits, 2).unwrap();
                return Some(Ok((1, Tok::Int(val), 1)));
            }
            self.chars.reset_peek();

            // tokenize([$0, $o, H | T], Line, Column, Scope, Tokens) when ?is_octal(H) ->
            if self.match_char('0') && self.match_char('o') && self.match_fn(&|ch| ch.is_digit(8)) {
                self.consume(2);
                let digits: String = self.chars.take_while_ref(|c| c.is_digit(8)).collect();
                self.col += digits.len() as u32;
                let val: BigInt = BigInt::from_str_radix(&digits, 8).unwrap();
                return Some(Ok((1, Tok::Int(val), 1)));
            }
            self.chars.reset_peek();

            // Comments

            // tokenize([$# | String], Line, Column, Scope, Tokens) ->
            if self.match_char('#') {
                self.consume(1);
                self.consume_to_eol();
                continue;
            }
            self.chars.reset_peek();

            // Sigils

            // tokenize([$~, S, H, H, H | T] = Original, Line, Column, Scope, Tokens) when ?is_quote(H), ?is_upcase(S) orelse ?is_downcase(S) ->
            if self.match_char('~') {
                if let Some(&s) = &self.chars.peek() {
                    if let Some(&h1) = &self.chars.peek() {
                        if let Some(&h2) = &self.chars.peek() {
                            if let Some(&h3) = &self.chars.peek() {
                                if s.is_ascii_alphabetic()
                                    && is_quote(&h1)
                                    && is_quote(&h2)
                                    && is_quote(&h3)
                                    && h1 == h2
                                    && h2 == h3
                                {
                                    self.consume(5);
                                    // TODO: extract_heredoc_with_interpolation...
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
            self.chars.reset_peek();

            // tokenize([$~, S, H | T] = Original, Line, Column, Scope, Tokens) when ?is_sigil(H), ?is_upcase(S) orelse ?is_downcase(S) ->
            if self.match_char('~') {
                if let Some(&s) = &self.chars.peek() {
                    if let Some(&h) = &self.chars.peek() {
                        if s.is_ascii_alphabetic() && is_sigil(&h) {
                            self.consume(3);
                            // TODO: elixir_interpolation:extract
                            return None;
                        }
                    }
                }
            }
            self.chars.reset_peek();

            // tokenize([$~, S, H | _] = Original, Line, Column, _Scope, Tokens) when ?is_upcase(S) orelse ?is_downcase(S) ->
            if self.match_char('~') {
                if let Some(&s) = &self.chars.peek() {
                    if let Some(&_h) = &self.chars.peek() {
                        if s.is_ascii_alphabetic() {
                            self.consume(3);
                            return Some(Err(LexicalError::InvalidSigil));
                        }
                    }
                }
            }
            self.chars.reset_peek();

            // Char tokens
            // tokenize([$?, $\\, H | T], Line, Column, Scope, Tokens) ->
            if self.match_char('?') && self.match_char('\\') {
                if let Some(&_h) = self.chars.peek() {
                    // TODO: elixir_interpolation:unescape_map(H)
                    // return Tok::Char..
                    return None;
                }
            }
            self.chars.reset_peek();

            // tokenize([$?, Char | T], Line, Column, Scope, Tokens) ->
            if self.match_char('?') {
                if let Some(&ch) = self.chars.peek() {
                    return Some(Ok((1, Tok::Char(ch), 1)));
                }
            }
            self.chars.reset_peek();

            // Heredocs

            // tokenize("\"\"\"" ++ T, Line, Column, Scope, Tokens) ->
            if self.match_char('"') && self.match_char('"') && self.match_char('"') {
                self.consume(3);
                // TODO: andle_heredocs(T, Line, Column, $", Scope, Tokens);
                return None;
            }
            self.chars.reset_peek();

            // tokenize("'''" ++ T, Line, Column, Scope, Tokens) ->
            if self.match_char('\'') && self.match_char('\'') && self.match_char('\'') {
                self.consume(3);
                // TODO: handle_heredocs(T, Line, Column, $', Scope, Tokens);
            }
            self.chars.reset_peek();

            // Strings

            // tokenize([$" | T], Line, Column, Scope, Tokens) ->
            if self.match_char('"') {
                self.consume(1);
                // TODO: handle_strings(T, Line, Column + 1, $", Scope, Tokens);
            }
            self.chars.reset_peek();

            // tokenize([$' | T], Line, Column, Scope, Tokens) ->
            if self.match_char('\'') {
                self.consume(1);
                // TODO: handle_strings(T, Line, Column + 1, $', Scope, Tokens);
            }
            self.chars.reset_peek();

            // Operator atoms

            // tokenize("...:" ++ Rest, Line, Column, Scope, Tokens) when ?is_space(hd(Rest)) ->
            if self.match_char('.')
                && self.match_char('.')
                && self.match_char('.')
                && self.match_char(':')
                && self.match_fn(&|c| is_space(c))
            {
                self.consume(4);
                return Some(Ok((1, Tok::KwIdentifier("...".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize("<<>>:" ++ Rest, Line, Column, Scope, Tokens) when ?is_space(hd(Rest)) ->
            if self.match_char('<')
                && self.match_char('<')
                && self.match_char('>')
                && self.match_char('>')
                && self.match_char(':')
                && self.match_fn(&|c| is_space(c))
            {
                self.consume(5);
                return Some(Ok((1, Tok::KwIdentifier("<<>>".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize("%{}:" ++ Rest, Line, Column, Scope, Tokens) when ?is_space(hd(Rest)) ->
            if self.match_char('%')
                && self.match_char('{')
                && self.match_char('}')
                && self.match_char(':')
                && self.match_fn(&|c| is_space(c))
            {
                self.consume(4);
                return Some(Ok((1, Tok::KwIdentifier("%{}".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize("%:" ++ Rest, Line, Column, Scope, Tokens) when ?is_space(hd(Rest)) ->
            if self.match_char('%') && self.match_char(':') && self.match_fn(&|c| is_space(c)) {
                self.consume(2);
                return Some(Ok((1, Tok::KwIdentifier("%".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize("{}:" ++ Rest, Line, Column, Scope, Tokens) when ?is_space(hd(Rest)) ->
            if self.match_char('{')
                && self.match_char('}')
                && self.match_char(':')
                && self.match_fn(&|c| is_space(c))
            {
                self.consume(3);
                return Some(Ok((1, Tok::KwIdentifier("{}".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize(":..." ++ Rest, Line, Column, Scope, Tokens) ->
            if self.match_char(':')
                && self.match_char('.')
                && self.match_char('.')
                && self.match_char('.')
            {
                self.consume(4);
                return Some(Ok((1, Tok::Atom("...".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize(":<<>>" ++ Rest, Line, Column, Scope, Tokens) ->
            if self.match_char(':')
                && self.match_char('<')
                && self.match_char('<')
                && self.match_char('>')
                && self.match_char('>')
            {
                self.consume(5);
                return Some(Ok((1, Tok::Atom("<<>>".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize(":%{}" ++ Rest, Line, Column, Scope, Tokens) ->
            if self.match_char(':')
                && self.match_char('%')
                && self.match_char('{')
                && self.match_char('}')
            {
                self.consume(4);
                return Some(Ok((1, Tok::Atom("%{}".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize(":%" ++ Rest, Line, Column, Scope, Tokens) ->
            if self.match_char(':') && self.match_char('%') {
                self.consume(2);
                return Some(Ok((1, Tok::Atom("%".to_string()), 1)));
            }
            self.chars.reset_peek();

            // tokenize(":{}" ++ Rest, Line, Column, Scope, Tokens) ->
            if self.match_char(':') && self.match_char('{') && self.match_char('}') {
                self.consume(3);
                return Some(Ok((1, Tok::Atom("{}".to_string()), 1)));
            }
            self.chars.reset_peek();

            // Three Token Operators

            // tokenize([$:, T1, T2, T3 | Rest], Line, Column, Scope, Tokens) when
            // ?unary_op3(T1, T2, T3); ?comp_op3(T1, T2, T3); ?and_op3(T1, T2, T3); ?or_op3(T1, T2, T3);
            // ?arrow_op3(T1, T2, T3); ?three_op(T1, T2, T3) ->
            if self.match_char(':') {
                if let Some(&t1) = self.chars.peek() {
                    if let Some(&t2) = self.chars.peek() {
                        if let Some(&t3) = self.chars.peek() {
                            if is_unary_op3(&t1, &t2, &t3)
                                || is_comp_op3(&t1, &t2, &t3)
                                || is_and_op3(&t1, &t2, &t3)
                                || is_or_op3(&t1, &t2, &t3)
                                || is_arrow_op3(&t1, &t2, &t3)
                                || is_three_op(&t1, &t2, &t3)
                            {
                                self.consume(4);
                                return Some(Ok((
                                    (self.col - 4) as usize,
                                    Tok::Atom([t1, t2, t3].into_iter().collect()),
                                    self.col as usize,
                                )));
                            }
                        }
                    }
                }
            }
            self.chars.reset_peek();

            // Two Token Operators
            // tokenize([$:, T1, T2 | Rest], Line, Column, Scope, Tokens) when
            //     ?comp_op2(T1, T2); ?rel_op2(T1, T2); ?and_op(T1, T2); ?or_op(T1, T2);
            //     ?arrow_op(T1, T2); ?in_match_op(T1, T2); ?two_op(T1, T2); ?list_op(T1, T2);
            // ?stab_op(T1, T2); ?type_op(T1, T2) ->
            if self.match_char(':') {
                if let Some(&t1) = self.chars.peek() {
                    if let Some(&t2) = self.chars.peek() {
                        if is_comp_op2(&t1, &t2)
                            || is_rel_op2(&t1, &t2)
                            || is_and_op(&t1, &t2)
                            || is_or_op(&t1, &t2)
                            || is_arrow_op(&t1, &t2)
                            || is_in_match_op(&t1, &t2)
                            || is_two_op(&t1, &t2)
                            || is_list_op(&t1, &t2)
                            || is_stab_op(&t1, &t2)
                            || is_type_op(&t1, &t2)
                        {
                            self.consume(3);
                            return Some(Ok((
                                (self.col - 3) as usize,
                                Tok::Atom([t1, t2].into_iter().collect()),
                                self.col as usize,
                            )));
                        }
                    }
                }
            }
            self.chars.reset_peek();

            //  ## Single Token Operators
            // tokenize([$:, T | Rest], Line, Column, Scope, Tokens) when
            //     ?at_op(T); ?unary_op(T); ?capture_op(T); ?dual_op(T); ?mult_op(T);
            // ?rel_op(T); ?match_op(T); ?pipe_op(T); T == $. ->
            if self.match_char(':') {
                if let Some(&t) = self.chars.peek() {
                    if is_at_op(&t)
                        || is_unary_op(&t)
                        || is_capture_op(&t)
                        || is_dual_op(&t)
                        || is_mult_op(&t)
                        || is_rel_op(&t)
                        || is_match_op(&t)
                        || is_pipe_op(&t)
                        || &t == &'.'
                    {
                        self.consume(2);
                        return Some(Ok((
                            (self.col - 2) as usize,
                            Tok::Atom([t].into_iter().collect()),
                            self.col as usize,
                        )));
                    }
                }
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

    fn match_fn(&mut self, f: &Fn(&char) -> bool) -> bool {
        self.chars.peek().map_or(false, |c| f(c))
    }
}

fn is_quote(c: &char) -> bool {
    c == &'\'' || c == &'"'
}

fn is_sigil(c: &char) -> bool {
    c == &'/'
        || c == &'<'
        || c == &'"'
        || c == &'\''
        || c == &'['
        || c == &'('
        || c == &'{'
        || c == &'|'
}

fn is_horizontal_space(s: &char) -> bool {
    s == &' ' || s == &'\t'
}

fn is_vertical_space(s: &char) -> bool {
    s == &'\r' || s == &'\n'
}

fn is_space(s: &char) -> bool {
    is_horizontal_space(s) || is_vertical_space(s)
}

fn is_comp_op2(t1: &char, t2: &char) -> bool {
    (t1 == &'=' && t2 == &'=') || (t1 == &'=' && t2 == &'~') || (t1 == &'!' && t2 == &'=')
}

fn is_rel_op2(t1: &char, t2: &char) -> bool {
    (t1 == &'<' && t2 == &'=') || (t1 == &'>' && t2 == &'~')
}

fn is_and_op(t1: &char, t2: &char) -> bool {
    (t1 == &'&' && t2 == &'&')
}

fn is_or_op(t1: &char, t2: &char) -> bool {
    (t1 == &'|' && t2 == &'|')
}

fn is_arrow_op(t1: &char, t2: &char) -> bool {
    (t1 == &'|' && t2 == &'>') || (t1 == &'~' && t2 == &'>') || (t1 == &'<' && t2 == &'~')
}

fn is_in_match_op(t1: &char, t2: &char) -> bool {
    (t1 == &'<' && t2 == &'-') || (t1 == &'\\' && t2 == &'\\')
}

fn is_two_op(t1: &char, t2: &char) -> bool {
    (t1 == &'<' && t2 == &'>') || (t1 == &'.' && t2 == &'.')
}

fn is_list_op(t1: &char, t2: &char) -> bool {
    (t1 == &'+' && t2 == &'+') || (t1 == &'-' && t2 == &'-')
}

fn is_stab_op(t1: &char, t2: &char) -> bool {
    (t1 == &'-' && t2 == &'>')
}

fn is_type_op(t1: &char, t2: &char) -> bool {
    (t1 == &':' && t2 == &':')
}

fn is_unary_op3(t1: &char, t2: &char, t3: &char) -> bool {
    t1 == &'~' && t2 == &'~' && t3 == &'~'
}

fn is_comp_op3(t1: &char, t2: &char, t3: &char) -> bool {
    (t1 == &'=' && t2 == &'=' && t3 == &'=') || (t1 == &'!' && t2 == &'=' && t3 == &'=')
}

fn is_and_op3(t1: &char, t2: &char, t3: &char) -> bool {
    t1 == &'&' && t2 == &'&' && t3 == &'&'
}

fn is_or_op3(t1: &char, t2: &char, t3: &char) -> bool {
    t1 == &'|' && t2 == &'|' && t3 == &'|'
}

fn is_arrow_op3(t1: &char, t2: &char, t3: &char) -> bool {
    (t1 == &'<' && t2 == &'<' && t3 == &'<')
        || (t1 == &'>' && t2 == &'>' && t3 == &'>')
        || (t1 == &'~' && t2 == &'>' && t3 == &'>')
        || (t1 == &'<' && t2 == &'<' && t3 == &'~')
        || (t1 == &'<' && t2 == &'~' && t3 == &'>')
        || (t1 == &'<' && t2 == &'|' && t3 == &'>')
}

fn is_three_op(t1: &char, t2: &char, t3: &char) -> bool {
    t1 == &'^' && t2 == &'^' && t3 == &'^'
}

fn is_at_op(t: &char) -> bool {
    t == &'@'
}

fn is_unary_op(t: &char) -> bool {
    t == &'!' || t == &'^'
}

fn is_capture_op(t: &char) -> bool {
    t == &'&'
}

fn is_dual_op(t: &char) -> bool {
    t == &'+' || t == &'-'
}

fn is_mult_op(t: &char) -> bool {
    t == &'*' || t == &'/'
}

fn is_rel_op(t: &char) -> bool {
    t == &'<' || t == &'>'
}

fn is_match_op(t: &char) -> bool {
    t == &'='
}

fn is_pipe_op(t: &char) -> bool {
    t == &'|'
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
    assert!(lexer.match_fn(&|ch| ch == &'a'));
}
