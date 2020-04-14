use core_tokens::Ident;
use std::collections::BTreeMap;

pub type Ty<'idt, 'tcx> = &'tcx Type<'idt>;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Type<'idt> {
    pub name: Ident<'idt>,
    pub ty: Variant<'idt>,
    pub size: usize,
    align: usize,
}

impl<'idt> Type<'idt> {
    pub const fn new(name: Ident<'idt>, ty: Variant<'idt>) -> Self {
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
pub enum Variant<'idt> {
    Primitive(Primitive),
    Trait {},
    Struct {
        fields: BTreeMap<Ident<'idt>, usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    I32,
}
