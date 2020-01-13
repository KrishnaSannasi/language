#![feature(const_generics)]

use lib_array_vec::ArrayDeque;

pub struct Peekable<I, T, const N: usize> {
    peek: ArrayDeque<T, N>,
    inner: I
}

impl<I, T, const N: usize> Peekable<I, T, N> {
    pub const fn new(inner: I) -> Self {
        Self {
            peek: ArrayDeque::new(), inner
        }
    }
}

#[cfg(feature="core-tokens")]
pub type PeekableLexer<'str, 'idt, L, const N: usize> = Peekable<L, core_tokens::Token<'str, 'idt>, N>;

#[cfg(feature="core-tokens")]
mod core_tokens_impl {
    use core_tokens::*;
    use super::*;
    
    impl<'str, 'idt, L: Lexer<'str, 'idt>, const N: usize> Peekable<L, Token<'str, 'idt>, N> {
        pub fn peek_token(&mut self, n: usize) -> impl Iterator<Item = &Token<'str, 'idt>> {
            assert!(n <= N, "tried to peek too far ahead");

            if let Some(new) = n.checked_sub(self.peek.len()) {
                for _ in 0..new {
                    if let Some(tok) = self.inner.parse_token() {
                        self.peek.push_back(tok)
                    } else {
                        break
                    }
                }
            }

            self.peek.iter().take(n)
        }
    }

    impl<'str, 'idt, L: Lexer<'str, 'idt>, const N: usize> Lexer<'str, 'idt> for Peekable<L, Token<'str, 'idt>, N> {
        fn parse_token(&mut self) -> Option<Token<'str, 'idt>> {
            let inner = &mut self.inner;
            self.peek.try_pop_front()
                .or_else(|| inner.parse_token())
        }

        fn parse_keyword(&mut self, kw: Option<Keyword>) -> Option<TokenValue<Keyword>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Keyword(ty), span, leading_whitespace })
                    if kw.is_none() || kw == Some(ty) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty, span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_keyword(kw)
            }
        }

        fn parse_ident(&mut self) -> Option<TokenValue<Ident<'idt>>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Ident(ty), span, leading_whitespace }) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty, span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_ident()
            }
        }

        fn parse_str(&mut self) -> Option<TokenValue<Str<'str>>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Str(ty), span, leading_whitespace }) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty, span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_str()
            }
        }

        fn parse_int(&mut self) -> Option<TokenValue<u128>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Int(ty), span, leading_whitespace }) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty, span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_int()
            }
        }

        fn parse_float(&mut self) -> Option<TokenValue<f64>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Float(ty), span, leading_whitespace }) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty, span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_float()
            }
        }

        fn parse_sym(&mut self, sym: Option<Symbol>) -> Option<TokenValue<Symbol>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Symbol(ty), span, leading_whitespace })
                    if sym.is_none() || sym == Some(ty) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty, span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_sym(sym)
            }
        }

        fn parse_grouping(
            &mut self,
            grouping: Option<(GroupPos, Grouping)>,
        ) -> Option<TokenValue<(GroupPos, Grouping)>> {
            match self.peek.front() {
                Some(&Token { ty: Type::Grouping(pos, gr), span, leading_whitespace })
                    if grouping.is_none() || grouping == Some((pos, gr)) => {
                        unsafe {
                            self.peek.pop_front_unchecked();
                        }

                        Some(TokenValue { ty: (pos, gr), span, leading_whitespace })
                    }
                Some(_) => None,
                None => self.inner.parse_grouping(grouping)
            }
        }
    }
} 