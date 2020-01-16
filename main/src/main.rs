use lib_arena::local::LocalUniqueArena;
use lib_intern::{Interner, Store};

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
    let hir_parser = impl_pass_hir::HirParser::new(lexer, context);

    let digest = impl_pass_mir::encode(hir_parser).expect("hi");

    println!("TARGETS:");
    for (i, mir) in digest.targets().iter().enumerate() {
        println!("{:3}: {:?}", i, mir);
    }
    
    println!("CODE:");
    for (i, mir) in digest.mir().iter().enumerate() {
        println!("{:3}: {:?}", i, mir);
    }
    
    // while let Some(hir_let) = hir_parser.parse() {
    //     dbg!(hir_let);
    // }
}
