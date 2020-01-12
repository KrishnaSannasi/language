use core_tokens::{Span, Str};

#[derive(Debug, PartialEq)]
pub struct HirNode<'str, 'idt, 'hir> {
    pub ty: Hir<'str, 'idt, 'hir>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Hir<'str, 'idt, 'hir> {
    Let {
        pat: Pattern<'str, 'idt>,
        ty: Option<Expr<'str, 'idt>>,
        value: Expr<'str, 'idt>,
    },
    Rec(&'hir mut Hir<'str, 'idt, 'hir>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Reference,
    Value,
}

#[derive(Debug, PartialEq)]
pub enum Pattern<'str, 'idt> {
    Literal(Literal<'str>),
    Ident(Str<'idt>, BindingMode),
    Tuple(Vec<Pattern<'str, 'idt>>),
}

#[derive(Debug, PartialEq)]
pub enum Expr<'str, 'idt> {
    Literal(Literal<'str>),
    Ident(Str<'idt>),
    Tuple(Vec<Pattern<'str, 'idt>>),
}

#[derive(Debug, PartialEq)]
pub enum Literal<'str> {
    Str(&'str str),
    Int(u128),
    Float(f64),
}
