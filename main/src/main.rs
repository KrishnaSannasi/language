use lib_thread_local::LazyThreadLocal;
use lib_arena::local::LocalUniqueArena;
use lib_intern::Intern;
use core_tokens::Lexer;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let file = std::fs::read_to_string(file).unwrap();
    // let arena = LocalUniqueArena::<_, 16>::new();
    let intern = Intern::new();

    let context = impl_lexer::Context {
        intern: &intern
    };
    
    let mut lexer = impl_lexer::Lexer::new(&file, context);

    while let Some(token) = lexer.parse() {
        println!("{:?}", token);
    }
}
