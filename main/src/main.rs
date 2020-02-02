use lib_arena::{cache::Cache, local::LocalUniqueArena};
use lib_intern::{Interner, Store};

fn main() {
    // let type_cache = Cache::new();

    let digest = {
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

        let context = impl_pass_mir::Context {
            // types: &type_cache 
        };
        impl_pass_mir::encode(hir_parser, context).expect("hi")
    };

    println!("CODE");

    let blocks = digest
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(i, block)| match block {
            Some(block) => Some((i, block)),
            None => None,
        });

    for (i, block) in blocks.clone() {
        println!("BLOCK({:3})", i);
        for (i, mir) in block.mir.iter().enumerate() {
            println!("{:3}: {:?}", i, mir);
        }
        println!()
    }

    println!("\nBLOCK DATA");

    for (i, block) in blocks {
        println!(
            "BLOCK({}):\
            \n\tparents: {:?}\
            \n\tchildren: {:?}",
            i, block.parents, block.children
        );
    }

    println!("\nTYPE INFO");

    for (i, ty) in digest.types.iter().enumerate() {
        println!("type_of {}: {:?}", i, ty);
    }

    println!("\nPROGRAM OUTPUT:\n");

    interp_mir::interpret(digest);

    // while let Some(hir_let) = hir_parser.parse() {
    //     dbg!(hir_let);
    // }
}
