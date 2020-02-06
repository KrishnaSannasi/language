use core_tokens::{Ident, Span, Str};

#[derive(Debug, PartialEq)]
pub struct Node<N> {
    pub val: N,
    pub span: Span,
}

pub type HirNode<'str, 'idt, 'hir> = Node<Hir<'str, 'idt, 'hir>>;
pub type Scope<'str, 'idt, 'hir> = Vec<Node<Hir<'str, 'idt, 'hir>>>;

#[derive(Debug, PartialEq)]
pub enum Hir<'str, 'idt, 'hir> {
    Let {
        pat: Node<Pattern<'str, 'idt>>,
        value: Node<Expr<'str, 'idt, 'hir>>,
    },
    If {
        if_branch: If<'str, 'idt, 'hir>,
        else_if_branches: Vec<If<'str, 'idt, 'hir>>,
        else_branch: Option<Box<Node<Scope<'str, 'idt, 'hir>>>>,
    },
    Mut {
        pat: Node<Pattern<'str, 'idt>>,
        value: Node<Expr<'str, 'idt, 'hir>>,
    },
    Print(Ident<'idt>),
    Scope(Scope<'str, 'idt, 'hir>),
    Loop(Scope<'str, 'idt, 'hir>),
    Rec(std::convert::Infallible, &'hir mut Hir<'str, 'idt, 'hir>),
    ControlFlow {
        ty: ControlFlowType,
        label: Option<Ident<'idt>>,
        val: Option<Expr<'str, 'idt, 'hir>>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ControlFlowType {
    Break,
    Continue,
}

#[derive(Debug, PartialEq)]
pub struct If<'str, 'idt, 'hir> {
    pub cond: Node<Expr<'str, 'idt, 'hir>>,
    pub branch: Node<Scope<'str, 'idt, 'hir>>,
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
pub enum Expr<'str, 'idt, 'hir> {
    Simple(Node<SimpleExpr<'str, 'idt>>),
    PreOp(Operator, Node<SimpleExpr<'str, 'idt>>),
    PostOp(Operator, Node<SimpleExpr<'str, 'idt>>),
    BinOp(
        Operator,
        Node<SimpleExpr<'str, 'idt>>,
        Node<SimpleExpr<'str, 'idt>>,
    ),
    Func {
        param: Ident<'idt>,
        body: &'hir mut Node<Hir<'str, 'idt, 'hir>>,
    },
    Tuple(Vec<Node<Pattern<'str, 'idt>>>),
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
}

#[derive(Debug, PartialEq)]
pub enum Literal<'str> {
    Str(Str<'str>),
    Int(u128),
    Float(f64),
    Bool(bool),
}
