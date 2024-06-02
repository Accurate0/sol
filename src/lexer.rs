use std::{
    fmt::{self, Display},
    ops::{Index, Range},
    str::Chars,
};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenKind {
    Comment,
    Identifier,
    Literal,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Add,
    Subtract,
    Multiply,
    Comma,
    Eq,
    Divide,
    Gt,
    Lt,
    Whitespace,
    EndOfLine,
    Not,

    EndOfFile,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[Range::<usize>::from(index)]
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            // "{:?} - <{}, {}>",
            "Token::new(TokenKind::{:?}, Span {{ start: {}, end: {} }})",
            self.kind, self.span.start, self.span.end
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn text<'a>(&self, input: &'a str) -> &'a str {
        &input[self.span]
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

pub struct Cursor<'a> {
    chars: Chars<'a>,
    current_consumed: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            current_consumed: 0,
        }
    }

    fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or('\0')
    }

    fn current(&self) -> usize {
        self.current_consumed
    }

    fn next(&mut self) -> Option<char> {
        self.current_consumed += 1;
        self.chars.next()
    }

    fn consume_until(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while !predicate(self.peek()) {
            self.next();
        }
    }

    fn is_start_of_identifier(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_in_identifier(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn consume_identifier(&mut self, initial: char) -> Token {
        let start = self.current() - 1;
        let mut identifier = String::new();
        identifier.push(initial);

        loop {
            let c = self.peek();
            if self.is_in_identifier(c) {
                identifier.push(c);
                self.next();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::Identifier,
            Span {
                start,
                end: self.current(),
            },
        )
    }

    fn consume_number(&mut self, initial: char) -> Token {
        let start = self.current() - 1;
        let mut number = String::new();
        number.push(initial);

        let mut is_floating = false;
        loop {
            let c = self.peek();
            if c.is_ascii_digit() {
                number.push(c);
                self.next();
            } else if c == '.' && !is_floating {
                is_floating = true;
                number.push(c);
                self.next();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::Literal,
            Span {
                start,
                end: self.current(),
            },
        )
    }

    fn consume_quoted_string(&mut self) -> Token {
        let start = self.current() - 1;
        let mut s = String::new();
        loop {
            let c = self.peek();
            if c == '"' {
                self.next();
                return Token::new(
                    TokenKind::Literal,
                    Span {
                        start,
                        end: self.current(),
                    },
                );
            }

            s.push(c);
            self.next();
        }
    }

    fn consume_comment_or_divide(&mut self) -> Token {
        let start = self.current() - 1;
        let token_kind = if self.peek() == '/' {
            self.next();
            self.consume_until(|c| c == '\n');
            TokenKind::Comment
        } else {
            TokenKind::Divide
        };

        Token::new(
            token_kind,
            Span {
                start,
                end: self.current(),
            },
        )
    }

    pub fn next_token(&mut self) -> Token {
        let next = self.next();
        if next.is_none() {
            return Token::new(
                TokenKind::EndOfFile,
                Span {
                    start: self.current(),
                    end: self.current(),
                },
            );
        }

        let next = next.unwrap();
        match next {
            '=' => Token::new(
                TokenKind::Eq,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '>' => Token::new(
                TokenKind::Gt,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '<' => Token::new(
                TokenKind::Lt,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '(' => Token::new(
                TokenKind::OpenParen,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            ')' => Token::new(
                TokenKind::CloseParen,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '{' => Token::new(
                TokenKind::OpenBrace,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '}' => Token::new(
                TokenKind::CloseBrace,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '+' => Token::new(
                TokenKind::Add,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '-' => Token::new(
                TokenKind::Subtract,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '*' => Token::new(
                TokenKind::Multiply,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            ',' => Token::new(
                TokenKind::Comma,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '!' => Token::new(
                TokenKind::Not,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '"' => self.consume_quoted_string(),
            '/' => self.consume_comment_or_divide(),
            c @ '0'..='9' => self.consume_number(c),
            ';' => Token::new(
                TokenKind::EndOfLine,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            c if c.is_ascii_whitespace() => {
                Token::new(TokenKind::Whitespace, Span { start: 0, end: 0 })
            }
            c if self.is_start_of_identifier(c) => self.consume_identifier(c),

            c => todo!("{}", c),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let token = self.cursor.next_token();
            match token.kind {
                TokenKind::EndOfFile => return None,
                TokenKind::Whitespace => continue,
                TokenKind::Comment => continue,
                _ => return Some(token),
            }
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            cursor: Cursor::new(contents.chars()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn not() {
        let input = r#"
            fn test() {
            let x = !true;
            }"#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 13, end: 15 }),
                Token::new(TokenKind::Identifier, Span { start: 16, end: 20 }),
                Token::new(TokenKind::OpenParen, Span { start: 20, end: 21 }),
                Token::new(TokenKind::CloseParen, Span { start: 21, end: 22 }),
                Token::new(TokenKind::OpenBrace, Span { start: 23, end: 24 }),
                Token::new(TokenKind::Identifier, Span { start: 37, end: 40 }),
                Token::new(TokenKind::Identifier, Span { start: 41, end: 42 }),
                Token::new(TokenKind::Eq, Span { start: 43, end: 44 }),
                Token::new(TokenKind::Not, Span { start: 45, end: 46 }),
                Token::new(TokenKind::Identifier, Span { start: 46, end: 50 }),
                Token::new(TokenKind::EndOfLine, Span { start: 50, end: 51 }),
                Token::new(TokenKind::CloseBrace, Span { start: 64, end: 65 }),
            ]
        );
    }

    #[test]
    fn complex_math() {
        let input = r#"
            fn test() {
                let z = (2 * 2) / ((3 - 4) * -2);
            }
        "#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 13, end: 15 }),
                Token::new(TokenKind::Identifier, Span { start: 16, end: 20 }),
                Token::new(TokenKind::OpenParen, Span { start: 20, end: 21 }),
                Token::new(TokenKind::CloseParen, Span { start: 21, end: 22 }),
                Token::new(TokenKind::OpenBrace, Span { start: 23, end: 24 }),
                Token::new(TokenKind::Identifier, Span { start: 41, end: 44 }),
                Token::new(TokenKind::Identifier, Span { start: 45, end: 46 }),
                Token::new(TokenKind::Eq, Span { start: 47, end: 48 }),
                Token::new(TokenKind::OpenParen, Span { start: 49, end: 50 }),
                Token::new(TokenKind::Literal, Span { start: 50, end: 51 }),
                Token::new(TokenKind::Multiply, Span { start: 52, end: 53 }),
                Token::new(TokenKind::Literal, Span { start: 54, end: 55 }),
                Token::new(TokenKind::CloseParen, Span { start: 55, end: 56 }),
                Token::new(TokenKind::Divide, Span { start: 57, end: 58 }),
                Token::new(TokenKind::OpenParen, Span { start: 59, end: 60 }),
                Token::new(TokenKind::OpenParen, Span { start: 60, end: 61 }),
                Token::new(TokenKind::Literal, Span { start: 61, end: 62 }),
                Token::new(TokenKind::Subtract, Span { start: 63, end: 64 }),
                Token::new(TokenKind::Literal, Span { start: 65, end: 66 }),
                Token::new(TokenKind::CloseParen, Span { start: 66, end: 67 }),
                Token::new(TokenKind::Multiply, Span { start: 68, end: 69 }),
                Token::new(TokenKind::Subtract, Span { start: 70, end: 71 }),
                Token::new(TokenKind::Literal, Span { start: 71, end: 72 }),
                Token::new(TokenKind::CloseParen, Span { start: 72, end: 73 }),
                Token::new(TokenKind::EndOfLine, Span { start: 73, end: 74 }),
                Token::new(TokenKind::CloseBrace, Span { start: 87, end: 88 }),
            ]
        )
    }

    #[test]
    fn math() {
        let input = r#"
            fn test() {
                let x = 2 + 3 / 2 * 3 - 1;
            }
        "#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 13, end: 15 }),
                Token::new(TokenKind::Identifier, Span { start: 16, end: 20 }),
                Token::new(TokenKind::OpenParen, Span { start: 20, end: 21 }),
                Token::new(TokenKind::CloseParen, Span { start: 21, end: 22 }),
                Token::new(TokenKind::OpenBrace, Span { start: 23, end: 24 }),
                Token::new(TokenKind::Identifier, Span { start: 41, end: 44 }),
                Token::new(TokenKind::Identifier, Span { start: 45, end: 46 }),
                Token::new(TokenKind::Eq, Span { start: 47, end: 48 }),
                Token::new(TokenKind::Literal, Span { start: 49, end: 50 }),
                Token::new(TokenKind::Add, Span { start: 51, end: 52 }),
                Token::new(TokenKind::Literal, Span { start: 53, end: 54 }),
                Token::new(TokenKind::Divide, Span { start: 55, end: 56 }),
                Token::new(TokenKind::Literal, Span { start: 57, end: 58 }),
                Token::new(TokenKind::Multiply, Span { start: 59, end: 60 }),
                Token::new(TokenKind::Literal, Span { start: 61, end: 62 }),
                Token::new(TokenKind::Subtract, Span { start: 63, end: 64 }),
                Token::new(TokenKind::Literal, Span { start: 65, end: 66 }),
                Token::new(TokenKind::EndOfLine, Span { start: 66, end: 67 }),
                Token::new(TokenKind::CloseBrace, Span { start: 80, end: 81 }),
            ]
        )
    }

    #[test]
    fn small_input() {
        let input = r#"
            const wow = 3;
            fn test() {}
        "#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 13, end: 18 }),
                Token::new(TokenKind::Identifier, Span { start: 19, end: 22 }),
                Token::new(TokenKind::Eq, Span { start: 23, end: 24 }),
                Token::new(TokenKind::Literal, Span { start: 25, end: 26 }),
                Token::new(TokenKind::EndOfLine, Span { start: 26, end: 27 }),
                Token::new(TokenKind::Identifier, Span { start: 40, end: 42 }),
                Token::new(TokenKind::Identifier, Span { start: 43, end: 47 }),
                Token::new(TokenKind::OpenParen, Span { start: 47, end: 48 }),
                Token::new(TokenKind::CloseParen, Span { start: 48, end: 49 }),
                Token::new(TokenKind::OpenBrace, Span { start: 50, end: 51 }),
                Token::new(TokenKind::CloseBrace, Span { start: 51, end: 52 }),
            ]
        );
    }

    #[test]
    fn larger_test() {
        let input = r#"
const wow = 3;

fn main(argv) {
    let x = 2;
    let y = true;
    print("test");
    print(1.3);


    print(x);
    print(2);

    test();
}

fn test(){
    if true {

    } else {
// comment
        print(2);
    }
}

fn new_function(arg1, arg2, arg3) {
{

    test ();
}
}"#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 1, end: 6 }),
                Token::new(TokenKind::Identifier, Span { start: 7, end: 10 }),
                Token::new(TokenKind::Eq, Span { start: 11, end: 12 }),
                Token::new(TokenKind::Literal, Span { start: 13, end: 14 }),
                Token::new(TokenKind::EndOfLine, Span { start: 14, end: 15 }),
                Token::new(TokenKind::Identifier, Span { start: 17, end: 19 }),
                Token::new(TokenKind::Identifier, Span { start: 20, end: 24 }),
                Token::new(TokenKind::OpenParen, Span { start: 24, end: 25 }),
                Token::new(TokenKind::Identifier, Span { start: 25, end: 29 }),
                Token::new(TokenKind::CloseParen, Span { start: 29, end: 30 }),
                Token::new(TokenKind::OpenBrace, Span { start: 31, end: 32 }),
                Token::new(TokenKind::Identifier, Span { start: 37, end: 40 }),
                Token::new(TokenKind::Identifier, Span { start: 41, end: 42 }),
                Token::new(TokenKind::Eq, Span { start: 43, end: 44 }),
                Token::new(TokenKind::Literal, Span { start: 45, end: 46 }),
                Token::new(TokenKind::EndOfLine, Span { start: 46, end: 47 }),
                Token::new(TokenKind::Identifier, Span { start: 52, end: 55 }),
                Token::new(TokenKind::Identifier, Span { start: 56, end: 57 }),
                Token::new(TokenKind::Eq, Span { start: 58, end: 59 }),
                Token::new(TokenKind::Identifier, Span { start: 60, end: 64 }),
                Token::new(TokenKind::EndOfLine, Span { start: 64, end: 65 }),
                Token::new(TokenKind::Identifier, Span { start: 70, end: 75 }),
                Token::new(TokenKind::OpenParen, Span { start: 75, end: 76 }),
                Token::new(TokenKind::Literal, Span { start: 76, end: 82 }),
                Token::new(TokenKind::CloseParen, Span { start: 82, end: 83 }),
                Token::new(TokenKind::EndOfLine, Span { start: 83, end: 84 }),
                Token::new(TokenKind::Identifier, Span { start: 89, end: 94 }),
                Token::new(TokenKind::OpenParen, Span { start: 94, end: 95 }),
                Token::new(TokenKind::Literal, Span { start: 95, end: 98 }),
                Token::new(TokenKind::CloseParen, Span { start: 98, end: 99 }),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 99,
                        end: 100
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 107,
                        end: 112
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 112,
                        end: 113
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 113,
                        end: 114
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 114,
                        end: 115
                    }
                ),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 115,
                        end: 116
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 121,
                        end: 126
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 126,
                        end: 127
                    }
                ),
                Token::new(
                    TokenKind::Literal,
                    Span {
                        start: 127,
                        end: 128
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 128,
                        end: 129
                    }
                ),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 129,
                        end: 130
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 136,
                        end: 140
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 140,
                        end: 141
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 141,
                        end: 142
                    }
                ),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 142,
                        end: 143
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 144,
                        end: 145
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 147,
                        end: 149
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 150,
                        end: 154
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 154,
                        end: 155
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 155,
                        end: 156
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 156,
                        end: 157
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 162,
                        end: 164
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 165,
                        end: 169
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 170,
                        end: 171
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 177,
                        end: 178
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 179,
                        end: 183
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 184,
                        end: 185
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 205,
                        end: 210
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 210,
                        end: 211
                    }
                ),
                Token::new(
                    TokenKind::Literal,
                    Span {
                        start: 211,
                        end: 212
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 212,
                        end: 213
                    }
                ),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 213,
                        end: 214
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 219,
                        end: 220
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 221,
                        end: 222
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 224,
                        end: 226
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 227,
                        end: 239
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 239,
                        end: 240
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 240,
                        end: 244
                    }
                ),
                Token::new(
                    TokenKind::Comma,
                    Span {
                        start: 244,
                        end: 245
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 246,
                        end: 250
                    }
                ),
                Token::new(
                    TokenKind::Comma,
                    Span {
                        start: 250,
                        end: 251
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 252,
                        end: 256
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 256,
                        end: 257
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 258,
                        end: 259
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 260,
                        end: 261
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 267,
                        end: 271
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 272,
                        end: 273
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 273,
                        end: 274
                    }
                ),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 274,
                        end: 275
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 276,
                        end: 277
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 278,
                        end: 279
                    }
                ),
            ]
        )
    }

    #[test]
    fn large_input() {
        let input = r#"
        const wow = 3;
        fn test(argv) {
            // this is a comment
            let a = "hello";
        }
        "#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 9, end: 14 }),
                Token::new(TokenKind::Identifier, Span { start: 15, end: 18 }),
                Token::new(TokenKind::Eq, Span { start: 19, end: 20 }),
                Token::new(TokenKind::Literal, Span { start: 21, end: 22 }),
                Token::new(TokenKind::EndOfLine, Span { start: 22, end: 23 }),
                Token::new(TokenKind::Identifier, Span { start: 32, end: 34 }),
                Token::new(TokenKind::Identifier, Span { start: 35, end: 39 }),
                Token::new(TokenKind::OpenParen, Span { start: 39, end: 40 }),
                Token::new(TokenKind::Identifier, Span { start: 40, end: 44 }),
                Token::new(TokenKind::CloseParen, Span { start: 44, end: 45 }),
                Token::new(TokenKind::OpenBrace, Span { start: 46, end: 47 }),
                Token::new(TokenKind::Identifier, Span { start: 93, end: 96 }),
                Token::new(TokenKind::Identifier, Span { start: 97, end: 98 }),
                Token::new(
                    TokenKind::Eq,
                    Span {
                        start: 99,
                        end: 100
                    }
                ),
                Token::new(
                    TokenKind::Literal,
                    Span {
                        start: 101,
                        end: 108
                    }
                ),
                Token::new(
                    TokenKind::EndOfLine,
                    Span {
                        start: 108,
                        end: 109
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 118,
                        end: 119
                    }
                ),
            ]
        );
    }
}
