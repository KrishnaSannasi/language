use core_tokens::{Span, Str, Ident};

#[derive(Debug, PartialEq)]
pub struct Node<N> {
    pub val: N,
    pub span: Span,
}

pub type HirNode<'str, 'idt, 'hir> = Node<Hir<'str, 'idt, 'hir>>;

#[derive(Debug, PartialEq)]
pub enum Hir<'str, 'idt, 'hir> {
    Let {
        pat: Node<Pattern<'str, 'idt>>,
        value: Node<Expr<'str, 'idt>>,
    },
    Print(Ident<'idt>),
    Scope(Vec<Node<Hir<'str, 'idt, 'hir>>>),
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
    Simple(Node<SimpleExpr<'str, 'idt>>),
    PreOp(Operator, Node<SimpleExpr<'str, 'idt>>),
    PostOp(Operator, Node<SimpleExpr<'str, 'idt>>),
    BinOp(Operator, Node<SimpleExpr<'str, 'idt>>, Node<SimpleExpr<'str, 'idt>>),
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
    Tuple(Vec<Node<Pattern<'str, 'idt>>>),
}

#[derive(Debug, PartialEq)]
pub enum Literal<'str> {
    Str(Str<'str>),
    Int(u128),
    Float(f64),
}
