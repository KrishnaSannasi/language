#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(Primitive),
    Inf(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Void {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    I32,
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        match *self {
            Type::Primitive(_) => true,
            _ => false,
        }
    }

    pub fn is_inference(&self) -> bool {
        match *self {
            Type::Inf(_) => true,
            _ => false,
        }
    }
}
