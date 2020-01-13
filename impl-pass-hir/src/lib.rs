use lib_arena::local::LocalUniqueArena;
use lib_peek::PeekableLexer;

use core_hir::{Hir, HirNode, Pattern, Expr, BindingMode, Literal};
use core_tokens::{Span, Lexer, kw, sym};

#[derive(Clone, Copy)]
pub struct Context<'str, 'idt, 'hir> {
    pub arena: &'hir LocalUniqueArena<HirNode<'str, 'idt, 'hir>, 16>,
}

pub struct HirParser<'str, 'idt, 'hir, L> {
    context: Context<'str, 'idt, 'hir>,
    lexer: PeekableLexer<'str, 'idt, L, 1>,
}

type Node<N> = <N as HasNode>::Node;
pub trait HasNode {
    type Node;
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> HasNode for HirParser<'str, 'idt, 'hir, L> {
    type Node = HirNode<'str, 'idt, 'hir>;
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> HirParser<'str, 'idt, 'hir, L> {
    pub fn new(lexer: L, context: Context<'str, 'idt, 'hir>) -> Self {
        Self { context, lexer: PeekableLexer::new(lexer), }
    }

    pub fn alloc(&self, node: HirNode<'str, 'idt, 'hir>) -> &'hir mut HirNode<'str, 'idt, 'hir> {
        self.context.arena.alloc(node)
    }

    pub fn parse(&mut self) -> Option<Node<Self>> {
        use core_tokens::Type;

        let token = self.lexer.peek_token(1).next()?;
        
        match token.ty {
            Type::Keyword(kw!(print)) => self.parse_print(),
            Type::Keyword(kw!(let)) => self.parse_let(),
            _ => None,
        }
    }

    pub fn parse_print(&mut self) -> Option<Node<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(print)))?;
        let ident = self.lexer.parse_ident()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(HirNode {
            span: start.span.to(end.span),
            ty: Hir::Print(ident.ty),
        })
    }

    pub fn parse_let(&mut self) -> Option<Node<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(let)))?;
        let ident = self.lexer.parse_ident()?;
        self.lexer.parse_sym(Some(sym!(=)))?;
        let value = self.lexer.parse_int()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(HirNode {
            span: start.span.to(end.span),
            ty: Hir::Let {
                pat: Pattern::Ident(ident.ty, BindingMode::Value),
                value: Expr::Literal(Literal::Int(value.ty)),
            }
        })
    }
}