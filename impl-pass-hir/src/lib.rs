use lib_arena::local::LocalUniqueArena;
use lib_peek::PeekableLexer;

use core_hir::{Hir, Node, Pattern, Expr, SimpleExpr, BindingMode, Literal};
use core_tokens::{Span, Lexer, kw, sym};

#[derive(Clone, Copy)]
pub struct Context<'str, 'idt, 'hir> {
    pub arena: &'hir LocalUniqueArena<Node<Hir<'str, 'idt, 'hir>>, 16>,
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
    type Node = Node<Hir<'str, 'idt, 'hir>>;
    type Expr = Node<Expr<'str, 'idt>>;
    type SimpleExpr = Node<SimpleExpr<'str, 'idt>>;
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> Iterator for HirParser<'str, 'idt, 'hir, L> {
    type Item = Node<Hir<'str, 'idt, 'hir>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}

impl<'str, 'idt, 'hir, L: Lexer<'str, 'idt>> HirParser<'str, 'idt, 'hir, L> {
    pub fn new(lexer: L, context: Context<'str, 'idt, 'hir>) -> Self {
        Self { context, lexer: PeekableLexer::new(lexer), }
    }

    pub fn alloc(&self, node: TNode<Self>) -> &'hir mut TNode<Self> {
        self.context.arena.alloc(node)
    }

    pub fn parse(&mut self) -> Option<TNode<Self>> {
        use core_tokens::{Type, Grouping, GroupPos};

        let token = self.lexer.peek_token(1).next()?;

        match token.ty {
            Type::Keyword(kw!(print)) => self.parse_print(),
            Type::Keyword(kw!(let)) => self.parse_let(),
            Type::Keyword(kw!(if)) => self.parse_if(),
            Type::Grouping(GroupPos::Start, Grouping::Curly) => {
                let scope = self.parse_scope()?;

                Some(Node {
                    val: Hir::Scope(scope.val),
                    span: scope.span,
                })
            },
            _ => None,
        }
    }

    pub fn parse_scope(&mut self) -> Option<Node<core_hir::Scope<'str, 'idt, 'hir>>> {
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

        Some(Node {
            span: start.span.to(end),
            val: inner,
        })
    }

    pub fn parse_print(&mut self) -> Option<TNode<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(print)))?;
        let ident = self.lexer.parse_ident()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(Node {
            span: start.span.to(end.span),
            val: Hir::Print(ident.ty),
        })
    }

    pub fn parse_let(&mut self) -> Option<TNode<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(let)))?;
        let ident = self.lexer.parse_ident()?;
        self.lexer.parse_sym(Some(sym!(=)))?;
        let value = self.parse_expr()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(Node {
            span: start.span.to(end.span),
            val: Hir::Let {
                value, pat: Node {
                    val: Pattern::Ident(ident.ty, BindingMode::Value),
                    span: ident.span,
                },
            }
        })
    }

    pub fn parse_if(&mut self) -> Option<TNode<Self>> {
        use core_tokens::{Token, Type, GroupPos, Grouping};

        let start = self.lexer.parse_keyword(Some(kw!(if)))?;
        let cond = self.parse_expr()?;
        let branch = self.parse_scope()?;

        let mut end_span = branch.span;

        let if_branch = core_hir::If { cond, branch };

        let mut else_if_branches = Vec::new();
        let mut else_branch = None;

        loop {
            let peek = self.lexer.peek_token(1).next();

            if let Some(Token { ty: Type::Keyword(kw!(else)), .. }) = peek {
                self.lexer.parse_keyword(Some(kw!(else)));
            } else {
                break
            };
            
            let peek = self.lexer.peek_token(1).next()?;

            match peek.ty {
                Type::Keyword(kw!(if)) => {
                    let cond = self.parse_expr()?;
                    let branch = self.parse_scope()?;
                    end_span = branch.span;

                    else_if_branches.push(core_hir::If { cond, branch });
                },
                Type::Grouping(GroupPos::Start, Grouping::Curly) => {
                    let branch = self.parse_scope()?;
                    end_span = branch.span;

                    else_branch = Some(Box::new(branch));

                    break
                },
                _ => return None
            }
        }

        Some(Node {
            span: start.span.to(end_span),
            val: Hir::If { if_branch, else_if_branches, else_branch }
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
                        | sym!(/)
                        | sym!(==)
                        | sym!(!=)
                        | sym!(>=)
                        | sym!(<=) => (),
                        _ => break 'simple
                    }

                    self.lexer.parse_token();

                    let next = self.parse_simple_expr()?;

                    return Some(Node {
                        span: expr.span.to(next.span),
                        val: Expr::BinOp(
                            core_hir::Operator::Symbol(sym),
                            expr,
                            next,
                        ),
                    })
                }
            }

            break
        }

        Some(Node {
            span: expr.span,
            val: Expr::Simple(expr),
        })
    }

    pub fn parse_simple_expr(&mut self) -> Option<TSimpleExpr<Self>> {
        use core_tokens::Type;

        let expr = self.lexer.peek_token(1).next()?;
        let span = expr.span;

        let val = match expr.ty {
            Type::Ident(ident) => SimpleExpr::Ident(ident),
            Type::Str(s) => SimpleExpr::Literal(Literal::Str(s)),
            Type::Int(x) => SimpleExpr::Literal(Literal::Int(x)),
            Type::Float(x) => SimpleExpr::Literal(Literal::Float(x)),
            _ => return None
        };

        self.lexer.parse_token();

        Some(Node { val, span, })
    }
}