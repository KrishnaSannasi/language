use core_tokens::Ident;
use core_hir::{Node, Hir, Pattern, BindingMode, Expr, SimpleExpr};
use core_mir::{Mir, Load, Reg, Type};
use lib_arena::cache::Cache;

use std::collections::{HashMap, HashSet};

use vec_utils::VecExt;

pub struct Context<'tcx> {
    pub types: &'tcx Cache<Type>,
}

pub struct Block<'tcx> {
    pub mir: Vec<Mir<'tcx>>,
    pub parents: HashSet<usize>,
    pub children: HashSet<usize>,
}

pub struct MirDigest<'tcx> {
    pub blocks: Vec<Option<Block<'tcx>>>,
    pub max_reg_count: u64,
}

struct Loop<'idt> {
    label: Option<Ident<'idt>>,
    start: usize,
    end: usize,
    exit: usize,
}

struct Encoder<'idt, 'tcx> {
    blocks: Vec<Block<'tcx>>,
    scopes: Vec<Scope<'idt, 'tcx>>,
    loop_stack: Vec<Loop<'idt>>,
    max_reg_count: u64,
    current_scope: usize,
    current_block: usize,
    context: Context<'tcx>,
}

#[derive(Default)]
pub struct Scope<'idt, 'tcx> {
    parent: usize,
    locals: HashMap<Ident<'idt>, Reg<'tcx>>,
}

pub fn encode<'tcx, 'str: 'hir, 'idt: 'hir, 'hir, H: IntoIterator<Item = Node<Hir<'str, 'idt, 'hir>>>>(
    hir: H,
    context: Context<'tcx>,
) -> Option<MirDigest<'tcx>> {
    let mut encoder = Encoder {
        blocks: Vec::new(),
        scopes: Vec::new(),
        loop_stack: Vec::new(),
        max_reg_count: 0,
        current_scope: 0,
        current_block: 0,
        context
    };

    encoder.blocks.push(Block {
        mir: Vec::new(),
        parents: HashSet::new(),
        children: HashSet::new(),
    });

    encoder.scopes.push(Scope::default());

    encode_iter(&mut encoder, hir)?;

    Some(MirDigest {
        blocks: encoder.blocks.map(Some),
        max_reg_count: encoder.max_reg_count,
    })
}

fn encode_iter<'tcx, 'str: 'hir, 'idt: 'hir, 'hir, H: IntoIterator<Item = Node<Hir<'str, 'idt, 'hir>>>>(
    encoder: &mut Encoder<'idt, 'tcx>,
    hir: H
) -> Option<()> {
    hir.into_iter().try_for_each(move |hir| encoder.encode(hir))
}

trait Encode<T> {
    type Output;

    #[must_use]
    fn encode(&mut self, value: T) -> Option<Self::Output>;
}

impl<'tcx, 'idt, 'str, 'hir> Encoder<'idt, 'tcx> {
    fn scopes(&self) -> impl Iterator<Item = &Scope<'idt, 'tcx>> {
        std::iter::successors(Some(self.current_scope), move |&scope| {
            if scope == 0 {
                None
            } else {
                Some(self.scopes[scope].parent)
            }
        })
        .map(move |scope| &self.scopes[scope])
    }
    
    fn get(&self, id: Ident<'idt>) -> Option<Reg<'tcx>> {
        self.scopes()
            .find_map(|scope| scope.locals.get(&id))
            .copied()
    }
    
    fn get_or_insert(&mut self, id: Ident<'idt>) -> Reg<'tcx> {
        if let Some(reg) = self.get(id) {
            reg
        } else {
            self.insert(id)
        }
    }
    
    fn insert(&mut self, id: Ident<'idt>) -> Reg<'tcx> {
        let scope = &mut self.scopes[self.current_scope];
        let reg = Reg(self.max_reg_count, None);
        scope.locals.insert(id, reg);
        self.max_reg_count += 1;
        reg
    }
    
    fn temp(&mut self) -> Reg<'tcx> {
        let reg = Reg(self.max_reg_count, None);
        self.max_reg_count += 1;
        reg
    }

    fn new_block(&mut self) -> usize {
        let target = self.blocks.len();

        self.blocks.push(Block {
            mir: Vec::new(),
            parents: HashSet::new(),
            children: HashSet::new(),
        });

        target
    }

    fn open_scope(&mut self) {
        let parent = self.current_scope;
        self.current_scope = self.scopes.len();

        self.scopes.push(Scope { parent, ..Scope::default() })
    }

    fn close_scope(&mut self) {
        self.current_scope = self.scopes[self.current_scope].parent;
    }

    fn jump(&mut self, from: usize, to: usize) {
        self.blocks[from].mir.push(Mir::Jump(to));
        self.blocks[to].parents.insert(from);
        self.blocks[from].children.insert(to);
    }

    fn branch(&mut self, cond: Reg<'tcx>, from: usize, to: usize) {
        self.blocks[from].mir.push(Mir::BranchTrue { cond, target: to });
        self.blocks[to].parents.insert(from);
        self.blocks[from].children.insert(to);
    }
}

impl<'tcx, 'idt, 'str, F> Encode<(Node<Expr<'str, 'idt>>, F)> for Encoder<'idt, 'tcx>
where
    F: FnOnce(&mut Self) -> Reg<'tcx>
{
    type Output = Reg<'tcx>;

    fn encode(&mut self, (value, to): (Node<Expr<'str, 'idt>>, F)) -> Option<Self::Output> {
        let reg;

        let mir = match value.val {
            Expr::PreOp(op, right) => todo!("preop"),
            Expr::PostOp(op, left) => todo!("postop"),
            Expr::Tuple(lit) => todo!("tuple"),
            Expr::Simple(simple) => {
                reg = to(self);
                Mir::LoadReg { to: reg, from: self.encode(simple)? }
            },
            Expr::BinOp(op, left, right) => {
                use core_hir::Operator;
                use core_tokens::sym;
                use core_mir::BinOpType;

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

                        _ => return None
                    }
                };
                
                reg = to(self);

                Mir::BinOp { op, out: reg, left, right }
            },
        };

        self.blocks[self.current_block].mir.push(mir);
        Some(reg)
    }
}

impl<'tcx, 'idt, 'str, 'hir> Encode<Vec<Node<Hir<'str, 'idt, 'hir>>>> for Encoder<'idt, 'tcx> {
    type Output = ();

    fn encode(&mut self, scope: Vec<Node<Hir<'str, 'idt, 'hir>>>) -> Option<Self::Output> {
        self.open_scope();
        encode_iter(self, scope)?;
        self.close_scope();
        Some(())
    }
}

impl<'tcx, 'idt, 'str, 'hir> Encode<Node<Hir<'str, 'idt, 'hir>>> for Encoder<'idt, 'tcx> {
    type Output = ();

    fn encode(&mut self, value: Node<Hir<'str, 'idt, 'hir>>) -> Option<Self::Output> {
        match value.val {
            Hir::Rec(infallible, _) => match infallible {}
            Hir::Scope(inner) => self.encode(inner)?,
            Hir::Loop(inner) => {
                let start = self.new_block();
                let end = self.new_block();
                let exit = self.new_block();
                self.loop_stack.push(Loop { label: None, start, end, exit });
                self.jump(self.current_block, start);
                self.jump(end, start);
                self.current_block = start;
                self.encode(inner)?;
                self.jump(self.current_block, end);
                self.loop_stack.pop();
            }
            Hir::ControlFlow {
                ty: core_hir::ControlFlowType::Break, label, val
            } => {
                assert!(label.is_none());
                assert!(val.is_none());

                let &Loop { exit, .. } = self.loop_stack.last().expect("Break can only be used inside a loop");

                self.jump(self.current_block, exit);
            }
            Hir::ControlFlow {
                ty: core_hir::ControlFlowType::Continue, label, val
            } => {
                todo!("continue")
            }
            Hir::Print(id) => {
                let print = self.get(id).map(Mir::Print)?;
                self.blocks[self.current_block].mir.push(print);
            }
            Hir::Let { pat, value } => {
                let to = |this: &mut Self| match pat.val {
                    Pattern::Literal(_) => {
                        unreachable!(r#"invalid "let" pattern, cannot bind to literals"#)
                    }
                    Pattern::Ident(_, BindingMode::Reference) => {
                        unreachable!(r#"invalid "let" pattern, cannot bind to variables by reference"#)
                    }
                    Pattern::Tuple(_) => {
                        unimplemented!(r#"invalid "let" pattern, tuples are not implemented"#)
                    }
                    Pattern::Ident(ident, BindingMode::Value) => {
                        this.insert(ident)
                    }
                };

                self.encode((value, to))?;
            }
            Hir::Mut { pat, value } => {
                let to = match pat.val {
                    Pattern::Literal(_) => {
                        unreachable!(r#"invalid "let" pattern, cannot bind to literals"#)
                    }
                    Pattern::Ident(_, BindingMode::Reference) => {
                        unreachable!(r#"invalid "let" pattern, cannot bind to variables by reference"#)
                    }
                    Pattern::Tuple(_) => {
                        unimplemented!(r#"invalid "let" pattern, tuples are not implemented"#)
                    }
                    Pattern::Ident(ident, BindingMode::Value) => {
                        self.get(ident)?
                    }
                };

                self.encode((value, |_this: &mut Self| to))?;
            }
            Hir::If { if_branch,  else_if_branches, else_branch, } => {
                let bb_start = self.new_block();
                let trailing_block = self.new_block();
                
                self.jump(self.current_block, bb_start);
                self.current_block = bb_start;

                let init_block = bb_start;
                
                let next_branch = std::iter::once(if_branch)
                    .chain(else_if_branches)
                    .try_fold(
                        self.new_block(),
                        |bb_if_branch, core_hir::If { cond, branch, }| {
                            let bb_next_branch = self.new_block();

                            let current_block = self.current_block;
                            
                            self.current_block = init_block;
                            let cond = self.encode((
                                cond,
                                |this: &mut Self| this.temp()
                            ))?;
                            self.branch(cond, self.current_block, bb_if_branch);

                            self.current_block = current_block;
                            
                            self.current_block = bb_if_branch;
                            self.encode(branch.val)?;
                            self.jump(self.current_block, trailing_block);

                            Some(bb_next_branch)
                        }
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

impl<'tcx, 'idt, 'str, 'hir> Encode<Node<SimpleExpr<'str, 'idt>>> for Encoder<'idt, 'tcx> {
    type Output = Reg<'tcx>;

    fn encode(&mut self, value: Node<SimpleExpr<'str, 'idt>>) -> Option<Self::Output> {
        match value.val {
            SimpleExpr::Ident(ident) => match self.get(ident) {
                Some(reg) => Some(reg),
                None => {
                    eprintln!("ERROR: detected an uninitialized variable: {:?} at {:?}", ident, value.span);
                    None
                }
            }
            SimpleExpr::Literal(lit) => {
                use core_hir::Literal;

                let to = self.temp();
                
                let from = match lit {
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
                    },
                };


                self.blocks[self.current_block].mir.push(Mir::Load { to, from });

                Some(to)
            }
        }
    }    
}