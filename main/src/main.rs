use lib_arena::{cache::Cache, local::LocalUniqueArena};
use lib_intern::{Interner, Store};

fn main() -> std::io::Result<()> {
    let _ = std::fs::create_dir("target_c");
    let _ = std::fs::create_dir("target_c/fragments");
    let _ = std::fs::create_dir("target_c/fragment_objects");

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
        let exprs = LocalUniqueArena::new();
        let context = impl_pass_hir::Context {
            arena: &arena,
            exprs: &exprs,
        };
        let hir_parser = impl_pass_hir::HirParser::new(lexer, context);

        impl_pass_mir::encode::write(hir_parser).expect("could not encode for some reason")
    };

    let ty_ctx = Cache::new();
    let ident = Interner::new();

    println!("CODE");
    println!("{}", digest);

    let types = impl_pass_mir::type_check::infer_types(
        &digest,
        impl_pass_mir::type_check::Context {
            ident: &ident,
            ty: &ty_ctx,
        },
    )
    .unwrap();

    println!("CODE (type checked)");
    println!("{}", digest);
    println!("\nBLOCK DATA");

    for (i, block) in digest.blocks().iter().enumerate() {
        println!(
            "BLOCK({}):\
            \n\tparents: {:?}\
            \n\tchildren: {:?}",
            i, block.meta.parents, block.meta.children
        );
    }

    println!("\nTYPE INFO");

    for (i, ty) in types.iter().enumerate() {
        println!("type_of {}: {:?}", i, ty);
    }

    println!("\nPROGRAM OUTPUT:\n");

    // interp_mir::interpret(digest);

    let file = std::fs::File::create("target_c/fragments/test.c")?;

    let ty_names = lib_intern::Interner::new();
    interp_mir::emit_c(digest, &file, &ty_names)?;

    let out = std::process::Command::new("gcc")
        .arg("-Iinc")
        .arg("-c")
        .arg("target_c/fragments/test.c")
        .arg("-o")
        .arg("target_c/fragment_objects/test.o")
        .arg("-O3")
        .stdout(std::process::Stdio::piped())
        .spawn()?
        .wait()?;
    std::process::Command::new("gcc")
        .arg("target_c/fragment_objects/test.o")
        .arg("-o")
        .arg("target_c/test.exe")
        .stdout(std::process::Stdio::piped())
        .spawn()?
        .wait()?;
    // std::process::Command::new("./target_c/test.exe")
    //     .stdout(std::process::Stdio::piped())
    //     .stderr(std::process::Stdio::piped())
    //     .spawn()?
    //     .wait()?;
    // while let Some(hir_let) = hir_parser.parse() {
    //     dbg!(hir_let);
    // }

    Ok(())
}
