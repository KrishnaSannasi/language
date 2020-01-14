use core_tokens::Ident;
use core_hir::{HirNode, Hir, Pattern, BindingMode, Expr, SimpleExpr};
use core_mir::{Mir, Load, Reg};

use std::collections::HashMap;

pub struct MirDigest {
    pub mir: Vec<Mir>,
    pub max_reg_count: u64,
}

#[derive(Default)]
struct Encoder<'idt> {
    mir: Vec<Mir>,
    scopes: Vec<Scope<'idt>>,
    max_reg_count: u64,
    current_scope: usize,
}

#[derive(Default)]
pub struct Scope<'idt> {
    parent: usize,
    locals: HashMap<Ident<'idt>, Reg>,
}

pub fn encode<'str: 'hir, 'idt: 'hir, 'hir, H: IntoIterator<Item = HirNode<'str, 'idt, 'hir>>>(hir: H) -> Option<MirDigest> {
    let mut encoder = Encoder::default();

    encoder.scopes.push(Scope::default());

    encode_iter(&mut encoder, hir)?;

    Some(MirDigest {
        mir: encoder.mir,
        max_reg_count: encoder.max_reg_count,
    })
}

fn encode_iter<'str: 'hir, 'idt: 'hir, 'hir, H: IntoIterator<Item = HirNode<'str, 'idt, 'hir>>>(
    encoder: &mut Encoder<'idt>,
    hir: H
) -> Option<()> {
    hir.into_iter().try_for_each(move |hir| encoder.encode(hir))
}

trait Encode<T> {
    type Output;

    fn encode(&mut self, value: T) -> Option<Self::Output>;
}

impl<'idt, 'str, 'hir> Encoder<'idt> {
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

    fn open_scope(&mut self) {
        let parent = self.current_scope;
        self.current_scope = self.scopes.len();

        self.scopes.push(Scope { parent, ..Scope::default() })
    }

    fn close_scope(&mut self) {
        self.current_scope = self.scopes[self.current_scope].parent;
    }
}

impl<'idt, 'str, 'hir> Encode<HirNode<'str, 'idt, 'hir>> for Encoder<'idt> {
    type Output = ();

    fn encode(&mut self, value: HirNode<'str, 'idt, 'hir>) -> Option<Self::Output> {
        match value.ty {
            Hir::Rec(infallible, _) => match infallible {}
            Hir::Print(id) => {
                let print = self.get(id).map(Mir::Print)?;
                self.mir.push(print);
            }
            Hir::Scope(inner) => {
                self.open_scope();
                encode_iter(self, inner)?;
                self.close_scope();
            }
            Hir::Let { pat, value } => {
                let to = |this: &mut Self| match pat {
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

                let mir = match value {
                    Expr::PreOp(op, right) => unreachable!("preop"),
                    Expr::PostOp(op, left) => unreachable!("postop"),
                    Expr::Simple(simple) => Mir::Load { to: to(self), from: self.encode(simple)? },
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
                                _ => return None
                            }
                        };
                        
                        Mir::BinOp { op, out: to(self), left, right }
                    },
                };

                self.mir.push(mir);
            }
        }
        
        Some(())
    }
}

impl<'idt, 'str, 'hir> Encode<SimpleExpr<'str, 'idt>> for Encoder<'idt> {
    type Output = Load;


    fn encode(&mut self, value: SimpleExpr<'str, 'idt>) -> Option<Self::Output> {
        match value {
            SimpleExpr::Ident(ident) => self.get(ident).map(Load::Reg),
            SimpleExpr::Literal(lit) => {
                use core_hir::Literal;
                
                match lit {
                    Literal::Str(s) => todo!(),
                    Literal::Float(x) => todo!(),
                    Literal::Int(x) => {
                        Some(if x < (1 << 8) {
                            Load::U8(x as _)
                        } else if x < (1 << 16) {
                            Load::U16(x as _)
                        } else if x < (1 << 32) {
                            Load::U32(x as _)
                        } else if x < (1 << 64) {
                            Load::U64(x as _)
                        } else {
                            Load::U128(x as _)
                        })
                    },
                }
            }
            SimpleExpr::Tuple(lit) => {
                todo!()
            }
        }
    }    
}