use core_tokens::*;
use lib_intern::{Interner, Store};
use std::mem::replace;

#[derive(Clone, Copy)]
pub struct Context<'str, 'idt> {
    pub intern: &'idt Interner,
    pub small_strings: &'str Interner,
    pub long_strings: &'str Store,
    pub max_small_string_size: usize,
}

pub struct Lexer<'input, 'str, 'idt> {
    ctx: Context<'str, 'idt>,
    input: &'input str,
    start: usize,
    leading_whitespace: Option<Span>,
}

impl<'input, 'str, 'idt> Lexer<'input, 'str, 'idt> {
    pub const fn new(input: &'input str, ctx: Context<'str, 'idt>) -> Self {
        Self {
            ctx,
            input,
            start: 0,
            leading_whitespace: None,
        }
    }
}

fn split(s: &str, mut f: impl FnMut(char) -> bool) -> (&str, &str) {
    let len = s.chars().take_while(|&c| f(c)).map(char::len_utf8).sum();

    s.split_at(len)
}

impl<'input, 'str, 'idt> Lexer<'input, 'str, 'idt> {
    fn alloc_str(&self, s: &str) -> Str<'str> {
        if s.len() < self.ctx.max_small_string_size {
            self.ctx.small_strings.insert(s).into()
        } else {
            self.ctx.long_strings.insert(s).into()
        }
    }

    fn parse_whitespace(&mut self) -> Option<Span> {
        if let leading_whitespace @ Some(_) = self.leading_whitespace.take() {
            return leading_whitespace;
        }

        if self.input.chars().next()?.is_whitespace()
            || self.input.starts_with("//")
            || self.input.starts_with("/*")
        {
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
            let len = self
                .input
                .chars()
                .zip(chars)
                .scan(&mut state, |&mut &mut ref mut state, (a, b)| match *state {
                    State::Whitespace => {
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
                    }

                    State::LineComment => {
                        if a == '\n' {
                            *state = State::Whitespace;
                            Some(1)
                        } else {
                            Some(a.len_utf8())
                        }
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

            Some(Span::new(start, end))
        } else {
            Some(Span::new(self.start, self.start))
        }
    }
}

impl<'input, 'str, 'idt> core_tokens::Lexer<'str, 'idt> for Lexer<'input, 'str, 'idt> {
    fn parse_token(&mut self) -> Option<Token<'str, 'idt>> {
        let leading_whitespace = self.parse_whitespace()?;

        let c = self.input.chars().next()?;

        if c == '_' || c.is_alphabetic() {
            let (ident, input) = split(self.input, |c| c == '_' || c.is_alphanumeric());
            self.input = input;

            let end = self.start + ident.len();
            let start = replace(&mut self.start, end);

            let ty = if let Ok(keyword) = ident.parse() {
                Type::Keyword(keyword)
            } else {
                Type::Ident(Ident::new(self.ctx.intern.insert(ident)))
            };

            Some(TokenValue {
                ty,
                leading_whitespace,
                span: Span::new(start, end),
            })
        } else if c.is_numeric() {
            let (int, input) = split(self.input, char::is_numeric);
            let init_input = self.input;
            self.input = input;

            #[allow(clippy::never_loop)]
            while self.input.starts_with('.') {
                const PERIOD_LEN: usize = 1;
                assert_eq!('.'.len_utf8(), PERIOD_LEN);

                let (dec, input) = split(&self.input[PERIOD_LEN..], char::is_numeric);

                if dec.is_empty() {
                    // prevent '0.', you can't have a trailing `.`
                    break;
                }

                self.input = input;

                let len = int.len() + dec.len() + PERIOD_LEN;
                let end = self.start + len;
                let start = replace(&mut self.start, end);
                let value = init_input[..len].parse().unwrap();

                return Some(TokenValue {
                    leading_whitespace,
                    span: Span::new(start, end),
                    ty: Type::Float(value),
                });
            }

            let end = self.start + int.len();
            let start = replace(&mut self.start, end);
            let value = int.parse().unwrap();

            Some(TokenValue {
                leading_whitespace,
                span: Span::new(start, end),
                ty: Type::Int(value),
            })
        } else if c == '"' {
            let len = self
                .input
                .chars()
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
                return None;
            }

            self.input = &input[1..];
            let end = self.start + len + 1;
            let start = replace(&mut self.start, end);

            Some(TokenValue {
                leading_whitespace,
                span: Span::new(start, end),
                ty: Type::Str(self.alloc_str(&s[1..])),
            })
        } else {
            use Type::Symbol;

            #[allow(clippy::never_loop)]
            while let Some(c) = self.input.get(0..2) {
                let ty = match c {
                    "=>" => Symbol(sym!(=>)),
                    "->" => Symbol(sym!(->)),
                    "<=" => Symbol(sym!(<=)),
                    ">=" => Symbol(sym!(>=)),
                    "==" => Symbol(sym!(==)),
                    "!=" => Symbol(sym!(!=)),
                    "::" => Symbol(sym!(::)),

                    _ => break,
                };

                let end = self.start + 2;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[2..];

                return Some(TokenValue {
                    ty,
                    span,
                    leading_whitespace,
                });
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

                    _ => break,
                };

                let end = self.start + 1;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[1..];

                return Some(TokenValue {
                    ty,
                    span,
                    leading_whitespace,
                });
            }

            None
        }
    }

    fn parse_keyword(&mut self, kw: Option<Keyword>) -> Option<TokenValue<Keyword>> {
        let leading_whitespace = self.parse_whitespace()?;

        let (ident, input) = split(self.input, |c| c == '_' || c.is_alphanumeric());

        let ty = ident.parse().ok()?;

        if kw.is_none() || kw == Some(ty) {
            self.input = input;

            let end = self.start + ident.len();
            let start = replace(&mut self.start, end);

            Some(TokenValue {
                ty,
                leading_whitespace,
                span: Span::new(start, end),
            })
        } else {
            self.leading_whitespace = Some(leading_whitespace);
            None
        }
    }

    fn parse_ident(&mut self) -> Option<TokenValue<Ident<'idt>>> {
        let leading_whitespace = self.parse_whitespace()?;

        let (ident, input) = split(self.input, |c| c == '_' || c.is_alphanumeric());

        if ident.is_empty() {
            self.leading_whitespace = Some(leading_whitespace);
            None
        } else if ident.parse::<Keyword>().is_err() {
            self.input = input;

            let end = self.start + ident.len();
            let start = replace(&mut self.start, end);

            Some(TokenValue {
                ty: Ident::new(self.ctx.intern.insert(ident)),
                leading_whitespace,
                span: Span::new(start, end),
            })
        } else {
            self.leading_whitespace = Some(leading_whitespace);
            None
        }
    }

    fn parse_str(&mut self) -> Option<TokenValue<Str<'str>>> {
        let leading_whitespace = self.parse_whitespace()?;

        if self.input.starts_with('"') {
            self.leading_whitespace = Some(leading_whitespace);
            return None;
        }

        let len = self
            .input
            .chars()
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
            return None;
        }

        self.input = &input[1..];
        let end = self.start + len + 1;
        let start = replace(&mut self.start, end);

        Some(TokenValue {
            leading_whitespace,
            span: Span::new(start, end),
            ty: self.alloc_str(&s[1..]),
        })
    }

    fn parse_int(&mut self) -> Option<TokenValue<u128>> {
        let leading_whitespace = self.parse_whitespace()?;

        let (int, input) = split(self.input, char::is_numeric);

        if int.is_empty() {
            self.leading_whitespace = Some(leading_whitespace);
            return None;
        }

        self.input = input;

        let end = self.start + int.len();
        let start = replace(&mut self.start, end);
        let value = int.parse().unwrap();

        Some(TokenValue {
            leading_whitespace,
            span: Span::new(start, end),
            ty: value,
        })
    }

    fn parse_float(&mut self) -> Option<TokenValue<f64>> {
        let leading_whitespace = self.parse_whitespace()?;

        let (int, input) = split(self.input, char::is_numeric);

        if int.is_empty() {
            self.leading_whitespace = Some(leading_whitespace);
            return None;
        }

        const PERIOD_LEN: usize = 1;
        assert_eq!('.'.len_utf8(), PERIOD_LEN);

        let (dec, input) = split(&input[PERIOD_LEN..], char::is_numeric);

        if dec.is_empty() {
            self.leading_whitespace = Some(leading_whitespace);
            return None;
        }

        let len = int.len() + dec.len() + PERIOD_LEN;
        let end = self.start + len;
        let start = replace(&mut self.start, end);
        let value = self.input[..len].parse().unwrap();
        self.input = input;

        Some(TokenValue {
            leading_whitespace,
            span: Span::new(start, end),
            ty: value,
        })
    }

    fn parse_sym(&mut self, sym: Option<Symbol>) -> Option<TokenValue<Symbol>> {
        let leading_whitespace = self.parse_whitespace()?;

        #[allow(clippy::never_loop)]
        while let Some(c) = self.input.get(0..2) {
            let ty = match c {
                "=>" => sym!(=>),
                "->" => sym!(->),
                "<=" => sym!(<=),
                ">=" => sym!(>=),
                "==" => sym!(==),
                "!=" => sym!(!=),
                "::" => sym!(::),

                _ => break,
            };

            if sym.is_none() || sym == Some(ty) {
                let end = self.start + 2;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[2..];

                return Some(TokenValue {
                    ty,
                    span,
                    leading_whitespace,
                });
            } else {
                break;
            }
        }

        #[allow(clippy::never_loop)]
        while let Some(c) = self.input.get(0..1) {
            let ty = match c {
                "+" => sym!(+),
                "-" => sym!(-),
                "*" => sym!(*),
                "/" => sym!(/),
                "%" => sym!(%),
                "." => sym!(.),
                "," => sym!(,),
                ":" => sym!(:),
                ";" => sym!(;),
                "#" => sym!(#),
                "$" => sym!($),
                "?" => sym!(?),
                "!" => sym!(!),
                "&" => sym!(&),
                "|" => sym!(|),
                "^" => sym!(^),
                "=" => sym!(=),
                ">" => sym!(>),
                "<" => sym!(<),

                "'" => core_tokens::Symbol::Tick,

                _ => break,
            };

            if sym.is_none() || sym == Some(ty) {
                let end = self.start + 1;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[1..];

                return Some(TokenValue {
                    ty,
                    span,
                    leading_whitespace,
                });
            } else {
                break;
            }
        }

        self.leading_whitespace = Some(leading_whitespace);

        None
    }

    fn parse_grouping(
        &mut self,
        grouping: Option<(GroupPos, Grouping)>,
    ) -> Option<TokenValue<(GroupPos, Grouping)>> {
        let leading_whitespace = self.parse_whitespace()?;

        #[allow(clippy::never_loop)]
        while let Some(c) = self.input.get(0..1) {
            let ty = match c {
                "(" => (GroupPos::Start, Grouping::Paren),
                ")" => (GroupPos::End, Grouping::Paren),
                "[" => (GroupPos::Start, Grouping::Square),
                "]" => (GroupPos::End, Grouping::Square),
                "{" => (GroupPos::Start, Grouping::Curly),
                "}" => (GroupPos::End, Grouping::Curly),

                _ => break,
            };

            if grouping.is_none() || grouping == Some(ty) {
                let end = self.start + 1;
                let start = replace(&mut self.start, end);
                let span = Span::new(start, end);
                self.input = &self.input[1..];

                return Some(TokenValue {
                    ty,
                    span,
                    leading_whitespace,
                });
            } else {
                break;
            }
        }

        self.leading_whitespace = Some(leading_whitespace);

        None
    }
}
