use lib_arena::local::LocalUniqueArena;
use lib_peek::PeekableLexer;

use core_hir::{
    BindingMode, ControlFlowType, Expr, Hir, Literal, Node, Parameter, Pattern, SimpleExpr,
};
use core_tokens::{kw, sym, Lexer};

#[derive(Clone, Copy)]
pub struct Context<'str, 'idt, 'hir> {
    pub arena: &'hir LocalUniqueArena<Node<Hir<'str, 'idt, 'hir>>, 16>,
    pub exprs: &'hir LocalUniqueArena<Node<Expr<'str, 'idt, 'hir>>, 16>,
}

pub struct HirParser<'str, 'idt, 'hir, L> {
    context: Context<'str, 'idt, 'hir>,
    lexer: PeekableLexer<'str, 'idt, L, 2>,
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
    type Expr = Node<Expr<'str, 'idt, 'hir>>;
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
        Self {
            context,
            lexer: PeekableLexer::new(lexer),
        }
    }

    pub fn alloc(&self, node: TNode<Self>) -> &'hir mut TNode<Self> {
        self.context.arena.alloc(node)
    }

    pub fn peek(&mut self) -> Option<&core_tokens::TokenValue<core_tokens::Type>> {
        self.lexer.peek_token(1).next()
    }

    pub fn peek_2(&mut self) -> Option<[&core_tokens::TokenValue<core_tokens::Type>; 2]> {
        let mut iter = self.lexer.peek_token(2);

        match (iter.next(), iter.next()) {
            (Some(a), Some(b)) => Some([a, b]),
            _ => None,
        }
    }

    pub fn parse(&mut self) -> Option<TNode<Self>> {
        use core_tokens::{GroupPos, Grouping, Type};

        let token = self.peek()?;

        match token.ty {
            Type::Keyword(kw!(print)) => self.parse_print(),
            Type::Keyword(kw!(let)) => self.parse_let(),
            Type::Keyword(kw!(loop)) => self.parse_loop(),
            Type::Keyword(kw!(break)) => self.parse_break(),
            Type::Ident(_) => self.parse_mut(),
            Type::Keyword(kw!(if)) => self.parse_if(),
            Type::Grouping(GroupPos::Start, Grouping::Curly) => {
                let scope = self.parse_scope()?;

                Some(Node {
                    val: Hir::Scope(scope.val),
                    span: scope.span,
                })
            }
            _ => None,
        }
    }

    pub fn parse_scope(&mut self) -> Option<Node<core_hir::Scope<'str, 'idt, 'hir>>> {
        use core_tokens::{GroupPos, Grouping, Type};

        let mut inner = Vec::new();

        let start = self
            .lexer
            .parse_grouping(Some((GroupPos::Start, Grouping::Curly)))?;
        let mut end;

        loop {
            let peek = self.lexer.peek_token(1).next()?;
            end = peek.span;

            if let Type::Grouping(GroupPos::End, Grouping::Curly) = peek.ty {
                self.lexer.parse_token();
                break;
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
                value,
                pat: Node {
                    val: Pattern::Ident(ident.ty, BindingMode::Value),
                    span: ident.span,
                },
            },
        })
    }

    pub fn parse_mut(&mut self) -> Option<TNode<Self>> {
        let ident = self.lexer.parse_ident()?;
        self.lexer.parse_sym(Some(sym!(=)))?;
        let value = self.parse_expr()?;
        let end = self.lexer.parse_sym(Some(sym!(;)))?;

        Some(Node {
            span: ident.span.to(end.span),
            val: Hir::Mut {
                value,
                pat: Node {
                    val: Pattern::Ident(ident.ty, BindingMode::Value),
                    span: ident.span,
                },
            },
        })
    }

    pub fn parse_break(&mut self) -> Option<TNode<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(break)))?;

        Some(Node {
            span: start.span,
            val: Hir::ControlFlow {
                ty: ControlFlowType::Break,
                label: None,
                val: None,
            },
        })
    }

    pub fn parse_if(&mut self) -> Option<TNode<Self>> {
        use core_tokens::{GroupPos, Grouping, Token, Type};

        let start = self.lexer.parse_keyword(Some(kw!(if)))?;
        let cond = self.parse_expr()?;
        let branch = self.parse_scope()?;

        let mut end_span = branch.span;

        let if_branch = core_hir::If { cond, branch };

        let mut else_if_branches = Vec::new();
        let mut else_branch = None;

        loop {
            let peek = self.lexer.peek_token(1).next();

            if let Some(Token {
                ty: Type::Keyword(kw!(else)),
                ..
            }) = peek
            {
                self.lexer.parse_keyword(Some(kw!(else)));
            } else {
                break;
            };

            let peek = self.lexer.peek_token(1).next()?;

            match peek.ty {
                Type::Keyword(kw!(if)) => {
                    self.lexer.parse_keyword(Some(kw!(if)));
                    let cond = self.parse_expr()?;
                    let branch = self.parse_scope()?;
                    end_span = branch.span;

                    else_if_branches.push(core_hir::If { cond, branch });
                }
                Type::Grouping(GroupPos::Start, Grouping::Curly) => {
                    let branch = self.parse_scope()?;
                    end_span = branch.span;

                    else_branch = Some(Box::new(branch));

                    break;
                }
                _ => return None,
            }
        }

        Some(Node {
            span: start.span.to(end_span),
            val: Hir::If {
                if_branch,
                else_if_branches,
                else_branch,
            },
        })
    }

    pub fn parse_loop(&mut self) -> Option<TNode<Self>> {
        let start = self.lexer.parse_keyword(Some(kw!(loop)))?;
        let block = self.parse_scope()?;

        Some(Node {
            span: start.span.to(block.span),
            val: Hir::Loop(block.val),
        })
    }

    pub fn parse_expr(&mut self) -> Option<TExpr<Self>> {
        use core_tokens::Type;

        return self.parse_function();

        if let Some(
            [core_tokens::TokenValue {
                ty: Type::Ident(_), ..
            }, core_tokens::TokenValue {
                ty: Type::Symbol(sym!(->)),
                ..
            }],
        ) = self.peek_2()
        {
            return self.parse_function();
        }

        let expr = self.parse_simple_expr()?;
        let expr = Node {
            span: expr.span,
            val: Expr::Simple(expr),
        };

        let peek = self.lexer.peek_token(1).next();

        #[allow(clippy::never_loop)]
        'simple: loop {
            if let Some(peek) = peek {
                if let Type::Symbol(sym) = peek.ty {
                    match sym {
                        sym!(+)
                        | sym!(-)
                        | sym!(*)
                        | sym!(/)
                        | sym!(==)
                        | sym!(!=)
                        | sym!(>=)
                        | sym!(<=)
                        | sym!(>)
                        | sym!(<) => (),
                        _ => break 'simple,
                    }

                    self.lexer.parse_token();

                    let next = self.parse_expr()?;
                    let expr = self.context.exprs.alloc(expr);
                    let next = self.context.exprs.alloc(next);

                    return Some(Node {
                        span: expr.span.to(next.span),
                        val: Expr::BinOp(core_hir::Operator::Symbol(sym), expr, next),
                    });
                }
            }

            break;
        }

        Some(expr)
    }

    pub fn parse_function(&mut self) -> Option<TExpr<Self>> {
        use core_tokens::{TokenValue, Type};

        let mut parameter_list = Vec::new();
        let mut span = None;

        while let Some(
            [TokenValue {
                ty: Type::Ident(_), ..
            }, TokenValue {
                ty: Type::Symbol(sym!(->)),
                ..
            }],
        ) = self.peek_2()
        {
            let name = self.lexer.parse_ident()?;
            self.lexer.parse_sym(Some(sym!(->)))?;
            span.get_or_insert(name.span);

            parameter_list.push(Parameter {
                name: name.ty,
                ty: None,
            });
        }

        let body = self.parse_bin_cmp()?;

        if let Some(span) = span {
            let body = self.context.exprs.alloc(body);

            Some(Node {
                span: span.to(body.span),
                val: Expr::Func {
                    parameter_list,
                    body,
                },
            })
        } else {
            Some(body)
        }
    }

    pub fn parse_bin_op(
        &mut self,
        mut is_valid_op: impl FnMut(core_tokens::Symbol) -> bool,
        mut next_level: impl FnMut(&mut Self) -> Option<TExpr<Self>>,
    ) -> Option<TExpr<Self>> {
        use core_tokens::Type;

        let mut first = next_level(self)?;

        loop {
            let peek = self.lexer.peek_token(1).next();

            if let Some(peek) = peek {
                if let Type::Symbol(sym) = peek.ty {
                    if is_valid_op(sym) {
                        self.lexer.parse_token();

                        let right = next_level(self)?;
                        let left = self.context.exprs.alloc(first);
                        let right = self.context.exprs.alloc(right);

                        first = Node {
                            span: left.span.to(right.span),
                            val: Expr::BinOp(core_hir::Operator::Symbol(sym), right, left),
                        };

                        continue;
                    }
                }
            }

            break Some(first);
        }
    }

    pub fn parse_bin_cmp(&mut self) -> Option<TExpr<Self>> {
        self.parse_bin_op(
            |sym| match sym {
                sym!(==) | sym!(!=) | sym!(>=) | sym!(<=) | sym!(>) | sym!(<) => true,
                _ => false,
            },
            Self::parse_bin_sum,
        )
    }

    pub fn parse_bin_sum(&mut self) -> Option<TExpr<Self>> {
        self.parse_bin_op(
            |sym| match sym {
                sym!(+) | sym!(-) => true,
                _ => false,
            },
            Self::parse_bin_prod,
        )
    }

    pub fn parse_bin_prod(&mut self) -> Option<TExpr<Self>> {
        self.parse_bin_op(
            |sym| match sym {
                sym!(+) | sym!(-) => true,
                _ => false,
            },
            Self::parse_func_app,
        )
    }

    pub fn parse_func_app(&mut self) -> Option<TExpr<Self>> {
        use smallvec::SmallVec;
        let mut name_args = SmallVec::<[_; 1]>::new();
        name_args.push(self.parse_basic_expr()?);
        let mut span = name_args[0].span;

        while let Some(arg) = self.parse_basic_expr() {
            span = span.to(arg.span);
            name_args.push(arg)
        }

        if name_args.len() == 1 {
            name_args.pop()
        } else {
            Some(Node {
                span,
                val: Expr::FuncApp {
                    name_args: name_args.into_vec(),
                },
            })
        }
    }

    pub fn parse_basic_expr(&mut self) -> Option<TExpr<Self>> {
        use core_tokens::{GroupPos, Grouping, Type};

        match self.parse_simple_expr() {
            Some(expr) => Some(Node {
                span: expr.span,
                val: Expr::Simple(expr),
            }),
            None => {
                let expr = self.peek()?;

                match expr.ty {
                    Type::Grouping(GroupPos::Start, Grouping::Curly) => {
                        let scope = self.parse_scope()?;

                        Some(Node {
                            span: scope.span,
                            val: Expr::Scope(scope.val),
                        })
                    }
                    Type::Grouping(GroupPos::Start, Grouping::Paren) => {
                        self.lexer
                            .parse_grouping(Some((GroupPos::Start, Grouping::Paren)));
                        let expr = self.parse_expr()?;
                        self.lexer
                            .parse_grouping(Some((GroupPos::End, Grouping::Paren)));

                        Some(expr)
                    }
                    _ => None,
                }
            }
        }
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
            Type::Keyword(kw!(true)) => SimpleExpr::Literal(Literal::Bool(true)),
            Type::Keyword(kw!(false)) => SimpleExpr::Literal(Literal::Bool(false)),
            _ => return None,
        };

        self.lexer.parse_token();

        Some(Node { val, span })
    }
}
