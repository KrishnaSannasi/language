use lib_intern::Str;

pub trait Lexer<'input, 'idt> {
    fn parse(&mut self) -> Option<Token<'input, 'idt>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'input, 'idt> {
    pub ty: Type<'input, 'idt>,
    pub leading_whitespace: Span,
    pub span: Span
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);

        Self { start, end }
    }

    pub const fn start(&self) -> usize {
        self.start
    }
    
    pub const fn end(&self) -> usize {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type<'input, 'idt> {
    Keyword(Keyword),
    Ident(Str<'idt>),
    Str(&'input str),
    Int(u128),
    Float(f64),
    Symbol(Symbol),
    Grouping(GroupPos, Grouping),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupPos {
    Start,
    End
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Grouping {
    Paren,
    Square,
    Curly
}

macro_rules! sym_gen {
    ($(($($sym:tt)*) => $sym_val:ident)* --- $($pathalogical:tt)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Symbol {
            $($sym_val,)*
            Tick
        }

        #[macro_export]
        macro_rules! sym {
        $(
            ($($sym)*) => { $crate::Symbol::$sym_val };
        )*
        $($pathalogical)*
        }
    }
}

macro_rules! kw_gen {
    ($($kw:ident => $kw_val:ident)* --- $($pathalogical:tt)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Keyword {
            $($kw_val),*
        }

        #[derive(Debug)]
        pub struct InvalidKeyword;

        impl std::str::FromStr for Keyword {
            type Err = InvalidKeyword;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($kw) => Ok($crate::Keyword::$kw_val),)*
                    _ => Err(InvalidKeyword),
                }
            }
        }

        #[macro_export]
        macro_rules! kw {
        $(
            ($kw) => { $crate::Keyword::$kw_val };
        )*
        $($pathalogical)*
        }
    }
}

sym_gen! {
    (=) => Assign

    (+) => Add
    (-) => Sub
    (*) => Mul
    (/) => Div
    (%) => Rem

    (.) => Dot
    (,) => Comma
    (:) => Colon
    (::) => DoubleColon
    (;) => SemiColon
    (#) => Pound
    ($) => Dollar
    (?) => Question
    
    (!) => Exclaim
    (&) => BitAnd
    (|) => BitOr
    (&&) => LogAnd
    (||) => LogOr
    (^) => Xor

    (->) => SimpleArrow
    (=>) => BoldArrow

    (<) => Less
    (>) => Greater
    (<=) => LessEqual
    (>=) => GreaterEqual
    (==) => Equal
    (!=) => NotEqual

    ---

    ($($tokens:tt)*) => { compile_error!(concat!("no known symbol: \"", stringify!($($tokens)*), "\"")) }
}

kw_gen! {
    let => Let
    mut => Mut

    match => Match
    loop => Loop
    
    break => Break
    continue => Continue
    return => Return
    
    if => If
    else => Else
    while => While
    
    static => Static
    comp => Comp
    
    struct => Struct
    enum => Enum
    union => Union
    trait => Trait

    pub => Pub
    mod => Mod
    ---

    ($($tokens:tt)*) => { compile_error!(concat!("no known keyword: \"", stringify!($($tokens)*), "\"")) }
}