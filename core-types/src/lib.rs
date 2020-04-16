use core_tokens::Ident;
use std::collections::BTreeMap;

pub type Ty<'idt, 'tcx> = &'tcx Type<'idt, 'tcx>;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Type<'idt, 'tcx> {
    pub name: Ident<'idt>,
    pub ty: Variant<'idt, 'tcx>,
    pub size: usize,
    align: usize,
}

impl<'idt, 'tcx> Type<'idt, 'tcx> {
    pub const fn new(name: Ident<'idt>, ty: Variant<'idt, 'tcx>) -> Self {
        Self {
            name,
            ty,
            align: 1,
            size: 0,
        }
    }

    pub fn align_to(self, align: usize) -> Self {
        assert!(align.is_power_of_two());
        Self { align, ..self }
    }

    pub fn with_size(self, size: usize) -> Self {
        Self { size, ..self }
    }

    pub const fn align(&self) -> usize {
        self.align
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variant<'idt, 'tcx> {
    Primitive(Primitive),
    Trait {},
    Struct {
        fields: Option<BTreeMap<Ident<'idt>, Ty<'idt, 'tcx>>>,
    },
    Function {
        captures: BTreeMap<Ident<'idt>, Ty<'idt, 'tcx>>,
        arguments: Vec<Ty<'idt, 'tcx>>,
        return_type: Ty<'idt, 'tcx>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    I32,
}
