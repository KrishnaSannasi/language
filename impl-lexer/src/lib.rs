use lib_intern::Intern;
use core_tokens::{Token, Span, Type, sym};
use std::mem::replace;

#[derive(Clone, Copy)]
pub struct Context<'idt> {
    pub intern: &'idt Intern
}

pub struct Lexer<'input, 'idt> {
    ctx: Context<'idt>,
    input: &'input str,
    start: usize,
}

impl<'input, 'idt> Lexer<'input, 'idt> {
    pub fn new(input: &'input str, ctx: Context<'idt>) -> Self {
        Self { ctx, input, start: 0, }
    }
}

fn split(s: &str, mut f: impl FnMut(char) -> bool) -> (&str, &str) {
    let len = s.chars().take_while(|&c| f(c)).map(char::len_utf8).sum();

    s.split_at(len)
}

impl<'input, 'idt> core_tokens::Lexer<'input, 'idt> for Lexer<'input, 'idt> {
    fn parse(&mut self) -> Option<Token<'input, 'idt>> {
        let c = self.input.chars().next()?;
        
        let leading_whitespace = if c.is_whitespace() || self.input.starts_with("//") || self.input.starts_with("/*") {
            #[derive(Debug, Clone, Copy, PartialEq)]
            enum Block {
                Normal,
                NewStart,
                NewEnd,
            }
            
            #[derive(Debug, Clone, Copy, PartialEq)]
            enum State {
                Whitespace,
                LineComment,
                BlockComment(u32, Block),
            };

            let mut state = State::Whitespace;

            let mut chars = self.input.chars();
            chars.next();
            let len = self.input.chars()
                .zip(chars)
                .scan(&mut state, |&mut &mut ref mut state, (a, b)| match *state {
                    State::Whitespace => 
                    if a == '/' && b == '/' {
                        *state = State::LineComment;
                        Some(1)
                    } else if a == '/' && b == '*' {
                        *state = State::BlockComment(0, Block::NewStart);
                        Some(1)
                    } else if a.is_whitespace() {
                        Some(a.len_utf8())
                    } else {
                        None
                    }

                    State::LineComment => if a == '\n' {
                        *state = State::Whitespace;
                        Some(1)
                    } else {
                        Some(a.len_utf8())
                    }
                    
                    State::BlockComment(depth, Block::Normal) => {
                        let block = match a {
                            '/' if b == '*' => Block::NewStart,
                            '*' if b == '/' => Block::NewEnd,
                            _ => Block::Normal,
                        };

                        *state = State::BlockComment(depth, block);

                        Some(a.len_utf8())
                    }
                    
                    State::BlockComment(depth, Block::NewStart) => {
                        debug_assert_eq!(a, '*');
                        *state = State::BlockComment(depth + 1, Block::Normal);
                        Some(a.len_utf8())
                    }
                    
                    State::BlockComment(depth, Block::NewEnd) => {
                        debug_assert_eq!(a, '/');
                        *state = if depth == 1 {
                            State::Whitespace
                        } else {
                            State::BlockComment(depth - 1, Block::Normal)
                        };

                        Some(a.len_utf8())
                    }
                })
                .sum();
            
            self.input = &self.input[len..];
            let end = self.start + len;

            let start = replace(&mut self.start, end);

            Span::new(start, end)
        } else {
            Span::new(self.start, self.start)
        };

        let c = self.input.chars().next()?;

        if c == '_' || c.is_alphabetic() {
            let (ident, input) = split(self.input, |c| c == '_' || c.is_alphanumeric());
            self.input = input;

            let end = self.start + ident.len();
            let start = replace(&mut self.start, end);

            let ty = if let Ok(keyword) = ident.parse() {
                Type::Keyword(keyword)
            } else {
                Type::Ident(self.ctx.intern.insert(ident))
            };

            Some(Token {
                ty, leading_whitespace,
                span: Span::new(start, end),
            })
        } else if c.is_numeric() {
            let (int, input) = split(self.input, char::is_numeric);
            let init_input = self.input;
            self.input = input;
            
            if self.input.starts_with('.') {
                const PERIOD_LEN: usize = 1;
                assert_eq!('.'.len_utf8(), PERIOD_LEN);

                let (dec, input) = split(&self.input[PERIOD_LEN..], char::is_numeric);
                self.input = input;

                let len = int.len() + dec.len() + PERIOD_LEN;
                let end = self.start + len;
                let start = replace(&mut self.start, end);
                let value = init_input[..len].parse().unwrap();

                Some(Token {
                    leading_whitespace,
                    span: Span::new(start, end),
                    ty: Type::Float(value)
                })
            } else {
                let end = self.start + int.len();
                let start = replace(&mut self.start, end);
                let value = int.parse().unwrap();

                Some(Token {
                    leading_whitespace,
                    span: Span::new(start, end),
                    ty: Type::Int(value)
                })
            }
        } else if c == '"' {
            let len = self.input.chars()
                .scan(true, |keep, c| {
                    if *keep {
                        *keep = false;
                        Some(c.len_utf8())
                    } else if c != '"' {
                        *keep = c == '\\';
                        Some(c.len_utf8())
                    } else {
                        None
                    }
                })
                .sum();
            
            let (s, input) = self.input.split_at(len);

            if !input.starts_with('"') {
                // this should detected the end quote if it is a valid string,
                // otherwise there was no end quote

                self.input = "";
                println!("ERROR: Detected an invalid quote");
                return None
            }
            
            self.input = &input[1..];
            let end = self.start + len + 1;
            let start = replace(&mut self.start, end);

            Some(Token {
                leading_whitespace,
                span: Span::new(start, end),
                ty: Type::Str(&s[1..])
            })
        } else {
            use core_tokens::{GroupPos, Grouping};
            use Type::Symbol;

            #[allow(clippy::never_loop)]
            while let Some(c) = self.input.get(0..2) {
                let ty = match c {
                    "=>" => Symbol(sym!(=>)),
                    "<=" => Symbol(sym!(<=)),
                    ">=" => Symbol(sym!(>=)),
                    "==" => Symbol(sym!(==)),
                    "!=" => Symbol(sym!(!=)),
                    "::" => Symbol(sym!(::)),
                    
                    _ => break
                };

                let end = self.start + 2;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[2..];

                return Some(Token { ty, span, leading_whitespace })
            }
            
            #[allow(clippy::never_loop)]
            while let Some(c) = self.input.get(0..1) {
                let ty = match c {
                    "(" => Type::Grouping(GroupPos::Start, Grouping::Paren),
                    ")" => Type::Grouping(GroupPos::End, Grouping::Paren),
                    "[" => Type::Grouping(GroupPos::Start, Grouping::Square),
                    "]" => Type::Grouping(GroupPos::End, Grouping::Square),
                    "{" => Type::Grouping(GroupPos::Start, Grouping::Curly),
                    "}" => Type::Grouping(GroupPos::End, Grouping::Curly),

                    "+" => Symbol(sym!(+)),
                    "-" => Symbol(sym!(-)),
                    "*" => Symbol(sym!(*)),
                    "/" => Symbol(sym!(/)),
                    "%" => Symbol(sym!(%)),
                    "." => Symbol(sym!(.)),
                    "," => Symbol(sym!(,)),
                    ":" => Symbol(sym!(:)),
                    ";" => Symbol(sym!(;)),
                    "#" => Symbol(sym!(#)),
                    "$" => Symbol(sym!($)),
                    "?" => Symbol(sym!(?)),
                    "!" => Symbol(sym!(!)),
                    "&" => Symbol(sym!(&)),
                    "|" => Symbol(sym!(|)),
                    "^" => Symbol(sym!(^)),
                    "=" => Symbol(sym!(=)),
                    ">" => Symbol(sym!(>)),
                    "<" => Symbol(sym!(<)),

                    "'" => Symbol(core_tokens::Symbol::Tick),
                    
                    _ => break
                };

                let end = self.start + 1;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[1..];

                return Some(Token { ty, span, leading_whitespace })
            }
            
            None
        }
    }
}