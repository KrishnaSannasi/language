use lib_arena::local::LocalUniqueArena;
use core_hir::{Hir, HirNode, Pattern, Expr, BindingMode, Literal};
use core_tokens::{Span, Lexer, kw, sym};

#[derive(Clone, Copy)]
pub struct Context<'str, 'idt, 'hir> {
    pub arena: &'hir LocalUniqueArena<HirNode<'str, 'idt, 'hir>, 16>,
}

pub struct HirParser<'str, 'idt, 'hir, L> {
    context: Context<'str, 'idt, 'hir>,
    lexer: L,
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> HirParser<'str, 'idt, 'hir, L> {
    pub fn new(lexer: L, context: Context<'str, 'idt, 'hir>) -> Self {
        Self { context, lexer, }
    }

    pub fn alloc(&self, node: HirNode<'str, 'idt, 'hir>) -> &'hir mut HirNode<'str, 'idt, 'hir> {
        self.context.arena.alloc(node)
    }

    pub fn parse_let(&mut self) -> Option<HirNode<'str, 'idt, 'hir>> {
        let start = self.lexer.parse_keyword(Some(kw!(let)))?;
        let ident = self.lexer.parse_ident()?;
        self.lexer.parse_sym(Some(sym!(:)))?;
        let ty = self.lexer.parse_ident()?;
        self.lexer.parse_sym(Some(sym!(=)))?;
        let value = self.lexer.parse_int()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(HirNode {
            span: Span::new(start.span.start(), end.span.end()),
            ty: Hir::Let {
                pat: Pattern::Ident(ident.ty, BindingMode::Value),
                ty: Some(Expr::Ident(ty.ty)),
                value: Expr::Literal(Literal::Int(value.ty)),
            }
        })
    }
}