use lib_thread_local::LazyThreadLocal;
use lib_arena::local::LocalUniqueArena;
use lib_intern::Intern;
use core_tokens::{Lexer, Type};

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let file = std::fs::read_to_string(file).unwrap();
    // let arena = LocalUniqueArena::<_, 16>::new();
    let intern = Intern::new();

    let context = impl_lexer::Context {
        intern: &intern
    };
    
    let mut lexer = impl_lexer::Lexer::new(&file, context);

    let mut kwds = 0;
    let mut idents = 0;
    let mut ints = 0;
    let mut floats = 0;
    let mut strs = 0;
    let mut groups = 0;
    let mut syms = 0;

    while let Some(token) = lexer.parse_token() {
        println!("{:?}", token);

        match token.ty {
            Type::Keyword(_) => kwds += 1,
            Type::Ident(_) => idents += 1,
            Type::Int(_) => ints += 1,
            Type::Float(_) => floats += 1,
            Type::Str(_) => strs += 1,
            Type::Grouping(_, _) => groups += 1,
            Type::Symbol(_) => syms += 1
        }
    }

    dbg!(kwds);
    dbg!(idents);
    dbg!(ints);
    dbg!(floats);
    dbg!(strs);
    dbg!(groups);
    dbg!(syms);

    dbg!(intern.len());
}
