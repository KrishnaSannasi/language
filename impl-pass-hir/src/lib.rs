use lib_arena::local::LocalUniqueArena;
use lib_peek::PeekableLexer;

use core_hir::{Hir, HirNode, Pattern, Expr, SimpleExpr, BindingMode, Literal};
use core_tokens::{Span, Lexer, kw, sym};

#[derive(Clone, Copy)]
pub struct Context<'str, 'idt, 'hir> {
    pub arena: &'hir LocalUniqueArena<HirNode<'str, 'idt, 'hir>, 16>,
}

pub struct HirParser<'str, 'idt, 'hir, L> {
    context: Context<'str, 'idt, 'hir>,
    lexer: PeekableLexer<'str, 'idt, L, 1>,
}

type TNode<N> = <N as HasNode>::Node;
type TExpr<N> = <N as HasNode>::Expr;
type TSimpleExpr<N> = <N as HasNode>::SimpleExpr;

pub trait HasNode {
    type Node;
    type Expr;
    type SimpleExpr;
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> HasNode for HirParser<'str, 'idt, 'hir, L> {
    type Node = HirNode<'str, 'idt, 'hir>;
    type Expr = Expr<'str, 'idt>;
    type SimpleExpr = SimpleExpr<'str, 'idt>;
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> Iterator for HirParser<'str, 'idt, 'hir, L> {
    type Item = HirNode<'str, 'idt, 'hir>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> HirParser<'str, 'idt, 'hir, L> {
    pub fn new(lexer: L, context: Context<'str, 'idt, 'hir>) -> Self {
        Self { context, lexer: PeekableLexer::new(lexer), }
    }

    pub fn alloc(&self, node: HirNode<'str, 'idt, 'hir>) -> &'hir mut HirNode<'str, 'idt, 'hir> {
        self.context.arena.alloc(node)
    }

    pub fn parse(&mut self) -> Option<TNode<Self>> {
        use core_tokens::{Type, Grouping, GroupPos};

        let token = self.lexer.peek_token(1).next()?;
        
        match token.ty {
            Type::Keyword(kw!(print)) => self.parse_print(),
            Type::Keyword(kw!(let)) => self.parse_let(),
            Type::Grouping(GroupPos::Start, Grouping::Curly) => self.parse_scope(),
            _ => None,
        }
    }

    pub fn parse_scope(&mut self) -> Option<TNode<Self>> {
        use core_tokens::{Type, Grouping, GroupPos};

        let mut inner = Vec::new();

        let start = self.lexer.parse_grouping(Some((GroupPos::Start, Grouping::Curly)))?;
        let mut end;

        loop {
            let peek = self.lexer.peek_token(1).next()?;
            end = peek.span;
            
            if let Type::Grouping(GroupPos::End, Grouping::Curly) = peek.ty {
                self.lexer.parse_token();
                break
            } else {
                inner.push(self.parse()?)
            }
        }

        Some(HirNode {
            span: start.span.to(end),
            ty: Hir::Scope(inner),
        })
    }

    pub fn parse_print(&mut self) -> Option<TNode<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(print)))?;
        let ident = self.lexer.parse_ident()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(HirNode {
            span: start.span.to(end.span),
            ty: Hir::Print(ident.ty),
        })
    }

    pub fn parse_let(&mut self) -> Option<TNode<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(let)))?;
        let ident = self.lexer.parse_ident()?;
        self.lexer.parse_sym(Some(sym!(=)))?;
        let value = self.parse_expr()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(HirNode {
            span: start.span.to(end.span),
            ty: Hir::Let {
                pat: Pattern::Ident(ident.ty, BindingMode::Value),
                value,
            }
        })
    }

    pub fn parse_expr(&mut self) -> Option<TExpr<Self>> {
        use core_tokens::Type;

        let expr = self.parse_simple_expr()?;

        let peek = self.lexer.peek_token(1).next();

        #[allow(clippy::never_loop)]
        'simple: loop {
            if let Some(peek) = peek {
                if let Type::Symbol(sym) = peek.ty {
                    match sym {
                        | sym!(+)
                        | sym!(-)
                        | sym!(*)
                        | sym!(/) => (),
                        _ => break 'simple
                    }

                    self.lexer.parse_token();

                    let next = self.parse_simple_expr()?;

                    return Some(Expr::BinOp(
                        core_hir::Operator::Symbol(sym),
                        expr,
                        next,
                    ))
                }
            }

            break
        }

        Some(Expr::Simple(expr))
    }

    pub fn parse_simple_expr(&mut self) -> Option<TSimpleExpr<Self>> {
        use core_tokens::Type;

        let expr = self.lexer.peek_token(1).next()?;

        let token = match expr.ty {
            Type::Ident(ident) => SimpleExpr::Ident(ident),
            Type::Str(s) => SimpleExpr::Literal(Literal::Str(s)),
            Type::Int(x) => SimpleExpr::Literal(Literal::Int(x)),
            Type::Float(x) => SimpleExpr::Literal(Literal::Float(x)),
            _ => return None
        };

        self.lexer.parse_token();

        Some(token)
    }
}