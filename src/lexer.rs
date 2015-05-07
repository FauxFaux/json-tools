/// A lexer for utf-8 encoded json data
pub struct Lexer<I: Iterator<Item=char>> {
    chars: I,
    prev_end: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    /// `{`
    CurlyOpen,
    /// `}`
    CurlyClose,

    /// `[`
    BracketOpen,
    /// `]`
    BracketClose,

    /// `:`
    Colon,
    /// `,`
    Comma,


    /// A json string , like `"foo"`
    String,
    /// `true`
    BooleanTrue,
    /// `false`
    BooleanFalse,

    /// any json number, like `1.24123` or `123`
    // NOTE: We can't do numbers with our simplified lexer as it would require
    // us to read a char just to see that it's not a number and thus the previous
    // tokens are to be returned. But we cannot peek without drastically complicating
    // our so far quite speedy implementation.
    // Number,
    /// `null`
    Null,
    
    /// The type of the token could not be identified.
    /// Should be removed if this lexer is ever to be feature complete
    Invalid,
}

/// A pair of indices into the character stream returned by our source 
/// iterator.
/// It is an exclusive range.
#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    /// Index of the first the unicode character
    pub first: u64,
    /// Index one past the last unicode character
    pub end: u64,
}

/// A lexical token, identifying its kind and span.
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    /// The exact type of the token
    pub kind: TokenType,
    /// The span allows to reference back into the source character stream 
    /// to obtain the string making up the token.
    pub span: Span,
}

impl<I> Lexer<I> where I: Iterator<Item=char> {
    /// Returns a new Lexer from a given character iterator.
    pub fn new(chars: I) -> Lexer<I> {
        Lexer {
            chars: chars,
            prev_end: 0,
        }
    }
}

// Identifies the state of the lexer
enum Mode {
    // String parse mode: bool = ignore_next
    String(bool),
    // `null` parse mode: buf, buf-index
    Null([char; 4], usize),
    // `true` parse mode
    True([char; 4], usize),
    // `false` parse mode
    False([char; 5], usize),

    SlowPath,
}

impl<I> Iterator for Lexer<I> 
                    where I: Iterator<Item=char> {
    type Item = Token;

    /// Lex the underlying character stream to generate tokens
    fn next(&mut self) -> Option<Token> {
        let mut t: TokenType = TokenType::Invalid;
        let mut lcid: u64 = 0;

        let mut first = 0;
        let prev_end = self.prev_end;

        let mut state = Mode::SlowPath;

        for c in self.chars.by_ref() {
            lcid += 1;

            let mut set_cursor = || {
                first = prev_end + lcid - 1;
            };

            match state {
                Mode::String(ref mut ign_next) => {
                    if *ign_next && (c == '"' || c == '\\') {
                        *ign_next = false;
                        continue;
                    }
                    match c {
                        '"' => {
                            t = TokenType::String;
                            break;
                        },
                        '\\' => {
                            *ign_next = true;
                            continue;    
                        },
                        _ => {
                            continue;
                        }
                    }
                },
                Mode::Null(ref mut b, ref mut i) => {
                    b[*i] = c;
                    if *i == 3 {
                        // we know b[0] is 'n'
                        if b[1] == 'u' && b[2] == 'l' && b[3] == 'l' {
                            t = TokenType::Null;
                        }
                        break;
                    } else {
                        *i += 1;
                        continue;
                    }
                },
                Mode::True(ref mut b, ref mut i) => {
                    b[*i] = c;
                    if *i == 3 {
                        // we know b[0] is 't'
                        if b[1] == 'r' && b[2] == 'u' && b[3] == 'e' {
                            t = TokenType::BooleanTrue;
                        }
                        break;
                    } else {
                        *i += 1;
                        continue;
                    }
                },
                Mode::False(ref mut b, ref mut i) => {
                    b[*i] = c;
                    if *i == 4 {
                        // we know b[0] is 'f'
                        if b[1] == 'a' && b[2] == 'l' && b[3] == 's' && b[4] == 'e' {
                            t = TokenType::BooleanFalse;
                        }
                        break;
                    } else {
                        *i += 1;
                        continue;
                    }
                },
                Mode::SlowPath => {
                    match c {
                        '{' => { t = TokenType::CurlyOpen; set_cursor(); break; },
                        '}' => { t = TokenType::CurlyClose; set_cursor(); break; },
                        '"' => {
                            state = Mode::String(false);
                            set_cursor();
                        },
                        'n' => {
                            state = Mode::Null([c, 'x', 'x', 'x'], 1);
                            set_cursor();
                        },
                        't' => {
                            state = Mode::True([c, 'x', 'x', 'x'], 1);
                            set_cursor();
                        },
                        'f' => {
                            state = Mode::False([c, 'x', 'x', 'x', 'x'], 1);
                            set_cursor();
                        },
                        '[' => { t = TokenType::BracketOpen; set_cursor(); break; },
                        ']' => { t = TokenType::BracketClose; set_cursor(); break; },
                        ':' => { t = TokenType::Colon; set_cursor(); break; },
                        ',' => { t = TokenType::Comma; set_cursor(); break; },
                        '\\' => {
                            // invalid
                            debug_assert_eq!(t, TokenType::Invalid);
                            set_cursor();
                            break
                        }
                        _ => {
                            // Everything here is considered whitespace, which is skipped
                        },
                    }// end single character match
                }// end case SlowPath
            }// end match state
        }// end for each character


        if lcid == 0 {
            None
        } else {
            self.prev_end = self.prev_end + lcid;
            Some(Token {
                kind: t,
                span: Span {
                    first: first,
                    end: self.prev_end
                }
            })
        }
    }
}