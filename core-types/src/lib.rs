use core_tokens::Ident;

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
            name, ty, align: 1, size: 0,
        }
    }

    pub fn align_to(self, align: usize) -> Self {
        assert!(align.is_power_of_two());
        Self { align, ..self }
    }

    pub fn with_size(self, size: usize) -> Self {
        assert!(size.is_power_of_two());
        Self { size, ..self }
    }

    pub const fn align(&self) -> usize {
        self.align
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variant<'idt, 'tcx> {
    Primitive(Primitive),
    Trait {

    },
    Struct {
        fields: Vec<Ty<'idt, 'tcx>>,
    }
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum Type {
//     Primitive(Primitive),
//     Inf(usize),
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Void {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    I32,
}

// impl Type {
//     pub fn is_primitive(&self) -> bool {
//         match *self {
//             Type::Primitive(_) => true,
//             _ => false,
//         }
//     }

//     pub fn is_inference(&self) -> bool {
//         match *self {
//             Type::Inf(_) => true,
//             _ => false,
//         }
//     }
// }
