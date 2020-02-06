use core_hir::{BindingMode, Expr, Hir, Literal, Node, Pattern, SimpleExpr};
use core_mir::{Load, Reg};
use core_tokens::Ident;

use std::collections::{HashMap, HashSet};

use super::*;

pub struct Context {
    // pub types: &'tcx Cache<Type>,
}

struct Loop<'idt> {
    label: Option<Ident<'idt>>,
    start: usize,
    end: usize,
    exit: usize,
}

#[derive(Default)]
struct Encoder<'idt> {
    blocks: Vec<Block>,
    scopes: Vec<Scope<'idt>>,
    loop_stack: Vec<Loop<'idt>>,
    max_reg_count: usize,
    current_scope: usize,
    current_block: usize,
}

#[derive(Default)]
pub struct Scope<'idt> {
    parent: usize,
    locals: HashMap<Ident<'idt>, Reg>,
}

pub fn write<
    'tcx,
    'str: 'hir,
    'idt: 'hir,
    'hir,
    H: IntoIterator<Item = Node<Hir<'str, 'idt, 'hir>>>,
>(
    hir: H,
) -> Option<StackFrame> {
    let mut encoder = Encoder::default();

    encoder.blocks.push(Block {
        instructions: Vec::new(),
        meta: BlockMeta {
            parents: HashSet::new(),
            children: HashSet::new(),
        }
    });

    encoder.scopes.push(Scope::default());

    encode_iter(&mut encoder, hir)?;

    StackFrame::new(
        encoder.blocks,
        FrameMeta {
            max_reg_count: encoder.max_reg_count,
        }
    )
}

fn encode_iter<
    'tcx,
    'str: 'hir,
    'idt: 'hir,
    'hir,
    H: IntoIterator<Item = Node<Hir<'str, 'idt, 'hir>>>,
>(
    encoder: &mut Encoder<'idt>,
    hir: H,
) -> Option<()> {
    hir.into_iter().try_for_each(move |hir| encoder.encode(hir))
}

trait Encode<T> {
    type Output;

    #[must_use]
    fn encode(&mut self, value: T) -> Option<Self::Output>;
}

impl<'tcx, 'idt, 'str, 'hir> Encoder<'idt> {
    fn scopes(&self) -> impl Iterator<Item = &Scope<'idt>> {
        std::iter::successors(Some(self.current_scope), move |&scope| {
            if scope == 0 {
                None
            } else {
                Some(self.scopes[scope].parent)
            }
        })
        .map(move |scope| &self.scopes[scope])
    }

    fn get(&self, id: Ident<'idt>) -> Option<Reg> {
        self.scopes()
            .find_map(|scope| scope.locals.get(&id))
            .copied()
    }

    fn get_or_insert(&mut self, id: Ident<'idt>) -> Reg {
        if let Some(reg) = self.get(id) {
            reg
        } else {
            self.insert(id)
        }
    }

    fn insert(&mut self, id: Ident<'idt>) -> Reg {
        let scope = &mut self.scopes[self.current_scope];
        let reg = Reg(self.max_reg_count);
        scope.locals.insert(id, reg);
        self.max_reg_count += 1;
        reg
    }

    fn temp(&mut self) -> Reg {
        let reg = Reg(self.max_reg_count);
        self.max_reg_count += 1;
        reg
    }

    fn new_block(&mut self) -> usize {
        let target = self.blocks.len();

        self.blocks.push(Block {
            instructions: Vec::new(),
            meta: BlockMeta {
                parents: HashSet::new(),
                children: HashSet::new(),
            }
        });

        target
    }

    fn open_scope(&mut self) {
        let parent = self.current_scope;
        self.current_scope = self.scopes.len();

        self.scopes.push(Scope {
            parent,
            ..Scope::default()
        })
    }

    fn close_scope(&mut self) {
        self.current_scope = self.scopes[self.current_scope].parent;
    }

    fn jump(&mut self, from: usize, to: usize) {
        self.blocks[from].instructions.push(Mir::Jump(to));
        self.blocks[to].meta.parents.insert(from);
        self.blocks[from].meta.children.insert(to);
    }

    fn branch(&mut self, cond: Reg, from: usize, to: usize) {
        self.blocks[from]
            .instructions
            .push(Mir::BranchTrue { cond, target: to });
        self.blocks[to].meta.parents.insert(from);
        self.blocks[from].meta.children.insert(to);
    }
}

impl<'tcx, 'idt, 'str, 'hir, F> Encode<(Node<Expr<'str, 'idt, 'hir>>, F)> for Encoder<'idt>
where
    F: FnOnce(&mut Self) -> Reg,
{
    type Output = Reg;

    fn encode(&mut self, (value, to): (Node<Expr<'str, 'idt, 'hir>>, F)) -> Option<Self::Output> {
        let reg;

        match value.val {
            Expr::PreOp(op, right) => todo!("preop"),
            Expr::PostOp(op, left) => todo!("postop"),
            Expr::Tuple(lit) => todo!("tuple"),
            Expr::Simple(simple) => {
                let to = to(self);
                self.encode((simple, to))
            }
            Expr::Func { param, body } => {
                todo!("func")
            }
            Expr::BinOp(op, left, right) => {
                use core_hir::Operator;
                use core_mir::BinOpType;
                use core_tokens::sym;

                let left = self.encode(left)?;
                let right = self.encode(right)?;

                let op = match op {
                    Operator::Keyword(_) => return None,
                    Operator::Symbol(op) => match op {
                        sym!(+) => BinOpType::Add,
                        sym!(-) => BinOpType::Sub,
                        sym!(*) => BinOpType::Mul,
                        sym!(/) => BinOpType::Div,

                        sym!(==) => BinOpType::Equal,
                        sym!(!=) => BinOpType::NotEqual,
                        sym!(>=) => BinOpType::GreaterThanOrEqual,
                        sym!(<=) => BinOpType::LessThanOrEqual,
                        sym!(>) => BinOpType::GreaterThan,
                        sym!(<) => BinOpType::LessThan,

                        _ => return None,
                    },
                };

                reg = to(self);

                self.blocks[self.current_block].instructions.push(Mir::BinOp {
                    op,
                    out: reg,
                    left,
                    right,
                });
                Some(reg)
            }
        }
    }
}

impl<'tcx, 'idt, 'str, 'hir> Encode<Vec<Node<Hir<'str, 'idt, 'hir>>>> for Encoder<'idt> {
    type Output = ();

    fn encode(&mut self, scope: Vec<Node<Hir<'str, 'idt, 'hir>>>) -> Option<Self::Output> {
        self.open_scope();
        encode_iter(self, scope)?;
        self.close_scope();
        Some(())
    }
}

impl<'tcx, 'idt, 'str, 'hir> Encode<Node<Hir<'str, 'idt, 'hir>>> for Encoder<'idt> {
    type Output = ();

    fn encode(&mut self, value: Node<Hir<'str, 'idt, 'hir>>) -> Option<Self::Output> {
        match value.val {
            Hir::Rec(infallible, _) => match infallible {},
            Hir::Scope(inner) => self.encode(inner)?,
            Hir::Loop(inner) => {
                let start = self.new_block();
                let end = self.new_block();
                let exit = self.new_block();
                self.loop_stack.push(Loop {
                    label: None,
                    start,
                    end,
                    exit,
                });
                self.jump(self.current_block, start);
                self.jump(end, start);
                self.current_block = start;
                self.encode(inner)?;
                self.jump(self.current_block, end);
                self.loop_stack.pop();
            }
            Hir::ControlFlow {
                ty: core_hir::ControlFlowType::Break,
                label,
                val,
            } => {
                assert!(label.is_none());
                assert!(val.is_none());

                let &Loop { exit, .. } = self
                    .loop_stack
                    .last()
                    .expect("Break can only be used inside a loop");

                self.jump(self.current_block, exit);
            }
            Hir::ControlFlow {
                ty: core_hir::ControlFlowType::Continue,
                label,
                val,
            } => todo!("continue"),
            Hir::Print(id) => {
                let print = self.get(id).map(Mir::Print)?;
                self.blocks[self.current_block].instructions.push(print);
            }
            Hir::Let { pat, value } => {
                let to = |this: &mut Self| match pat.val {
                    Pattern::Literal(_) => {
                        unreachable!(r#"invalid "let" pattern, cannot bind to literals"#)
                    }
                    Pattern::Ident(_, BindingMode::Reference) => unreachable!(
                        r#"invalid "let" pattern, cannot bind to variables by reference"#
                    ),
                    Pattern::Tuple(_) => {
                        unimplemented!(r#"invalid "let" pattern, tuples are not implemented"#)
                    }
                    Pattern::Ident(ident, BindingMode::Value) => this.insert(ident),
                };

                self.encode((value, to))?;
            }
            Hir::Mut { pat, value } => {
                let to = match pat.val {
                    Pattern::Literal(_) => {
                        unreachable!(r#"invalid "let" pattern, cannot bind to literals"#)
                    }
                    Pattern::Ident(_, BindingMode::Reference) => unreachable!(
                        r#"invalid "let" pattern, cannot bind to variables by reference"#
                    ),
                    Pattern::Tuple(_) => {
                        unimplemented!(r#"invalid "let" pattern, tuples are not implemented"#)
                    }
                    Pattern::Ident(ident, BindingMode::Value) => self.get(ident)?,
                };

                self.encode((value, |_this: &mut Self| to))?;
            }
            Hir::If {
                if_branch,
                else_if_branches,
                else_branch,
            } => {
                let bb_start = self.new_block();
                let trailing_block = self.new_block();

                self.jump(self.current_block, bb_start);
                self.current_block = bb_start;

                let init_block = bb_start;

                let next_branch = std::iter::once(if_branch)
                    .chain(else_if_branches)
                    .try_fold(
                        self.new_block(),
                        |bb_if_branch, core_hir::If { cond, branch }| {
                            let bb_next_branch = self.new_block();

                            let current_block = self.current_block;

                            self.current_block = init_block;
                            let cond = self.encode((cond, |this: &mut Self| this.temp()))?;
                            self.branch(cond, self.current_block, bb_if_branch);

                            self.current_block = current_block;

                            self.current_block = bb_if_branch;
                            self.encode(branch.val)?;
                            self.jump(self.current_block, trailing_block);

                            Some(bb_next_branch)
                        },
                    )?;

                self.jump(init_block, next_branch);

                self.current_block = next_branch;
                if let Some(branch) = else_branch {
                    self.encode(branch.val)?;
                }
                self.jump(self.current_block, trailing_block);

                self.current_block = trailing_block;
            }
        }

        Some(())
    }
}

impl<'idt, 'str> Encode<Node<SimpleExpr<'str, 'idt>>> for Encoder<'idt> {
    type Output = Reg;

    fn encode(&mut self, value: Node<SimpleExpr<'str, 'idt>>) -> Option<Self::Output> {
        match value.val {
            SimpleExpr::Literal(lit) => {
                let to = self.temp();
                self.encode((lit, to))
            }
            SimpleExpr::Ident(ident) => match self.get(ident) {
                Some(from) => Some(from),
                None => {
                    eprintln!(
                        "ERROR: detected an uninitialized variable: {:?} at {:?}",
                        ident, value.span
                    );
                    None
                }
            },
        }
    }
}

impl<'idt, 'str> Encode<(Node<SimpleExpr<'str, 'idt>>, Reg)> for Encoder<'idt> {
    type Output = Reg;

    fn encode(&mut self, (value, to): (Node<SimpleExpr<'str, 'idt>>, Reg)) -> Option<Self::Output> {
        match value.val {
            SimpleExpr::Literal(lit) => self.encode((lit, to)),
            SimpleExpr::Ident(ident) => match self.get(ident) {
                Some(from) => {
                    self.blocks[self.current_block]
                        .instructions
                        .push(Mir::LoadReg { to, from });
                    Some(to)
                }
                None => {
                    eprintln!(
                        "ERROR: detected an uninitialized variable: {:?} at {:?}",
                        ident, value.span
                    );
                    None
                }
            },
        }
    }
}

impl<'idt, 'str> Encode<(Literal<'str>, Reg)> for Encoder<'idt> {
    type Output = Reg;

    fn encode(&mut self, (value, to): (Literal<'str>, Reg)) -> Option<Self::Output> {
        let from = match value {
            Literal::Str(s) => todo!("str"),
            Literal::Float(x) => todo!("float"),
            Literal::Bool(x) => Load::Bool(x),
            Literal::Int(x) => {
                if x < (1 << 8) {
                    Load::U8(x as _)
                } else if x < (1 << 16) {
                    Load::U16(x as _)
                } else if x < (1 << 32) {
                    Load::U32(x as _)
                } else if x < (1 << 64) {
                    Load::U64(x as _)
                } else {
                    Load::U128(x as _)
                }
            }
        };

        self.blocks[self.current_block]
            .instructions
            .push(Mir::Load { to, from });

        Some(to)
    }
}
