use core_tokens::{Lexer, Type};
use lib_arena::local::LocalUniqueArena;
use lib_intern::{Interner, Store};
use lib_thread_local::LazyThreadLocal;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let file = std::fs::read_to_string(file).unwrap();
    
    let intern = Interner::new();
    let small_strings = Interner::new();
    let long_strings = Store::new();

    let context = impl_lexer::Context {
        intern: &intern,
        small_strings: &small_strings,
        long_strings: &long_strings,
        max_small_string_size: 64,
    };

    let lexer = impl_lexer::Lexer::new(&file, context);

    let arena = LocalUniqueArena::new();
    let context = impl_pass_hir::Context { arena: &arena };
    let mut hir_parser = impl_pass_hir::HirParser::new(lexer, context);
    
    while let Some(hir_let) = hir_parser.parse() {
        dbg!(hir_let);
    }
}
