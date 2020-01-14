use core_tokens::{Span, Str, Ident};

#[derive(Debug, PartialEq)]
pub struct HirNode<'str, 'idt, 'hir> {
    pub ty: Hir<'str, 'idt, 'hir>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Hir<'str, 'idt, 'hir> {
    Let {
        pat: Pattern<'str, 'idt>,
        value: Expr<'str, 'idt>,
    },
    Print(Ident<'idt>),
    Scope(Vec<HirNode<'str, 'idt, 'hir>>),
    Rec(std::convert::Infallible, &'hir mut Hir<'str, 'idt, 'hir>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Reference,
    Value,
}

#[derive(Debug, PartialEq)]
pub enum Pattern<'str, 'idt> {
    Literal(Literal<'str>),
    Ident(Ident<'idt>, BindingMode),
    Tuple(Vec<Pattern<'str, 'idt>>),
}

#[derive(Debug, PartialEq)]
pub enum Expr<'str, 'idt> {
    Simple(SimpleExpr<'str, 'idt>),
    PreOp(Operator, SimpleExpr<'str, 'idt>),
    PostOp(Operator, SimpleExpr<'str, 'idt>),
    BinOp(Operator, SimpleExpr<'str, 'idt>, SimpleExpr<'str, 'idt>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Symbol(core_tokens::Symbol),
    Keyword(core_tokens::Keyword),
}

#[derive(Debug, PartialEq)]
pub enum SimpleExpr<'str, 'idt> {
    Literal(Literal<'str>),
    Ident(Ident<'idt>),
    Tuple(Vec<Pattern<'str, 'idt>>),
}

#[derive(Debug, PartialEq)]
pub enum Literal<'str> {
    Str(Str<'str>),
    Int(u128),
    Float(f64),
}
