use clap::Parser;
use clio::InputPath;
use redox_codegen::{
    llvm::{LLVMCodegenBackend, LLVMContext},
    CodegenBackend,
};
use std::io::Read;

use redox_ir_generator::{IrGenerator, ModuleOps};
use redox_lexer::{LexerTrait, Token};
use redox_parser::Parser as RedoxParser;
use redox_type_checker::TypeChecker;

#[derive(Parser, Debug, Clone)]
struct Args {
    input: InputPath,
    #[clap(short, long, default_value = "0")]
    verbose: u32,
}

fn main() {
    let args = Args::parse();

    let mut contents = String::new();
    args.input
        .open()
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    if args.verbose >= 3 {
        eprintln!("Invalid verbose level (0-3)");
        std::process::exit(1);
    }

    if args.verbose >= 1 {
        eprintln!("Redox Compiler");
        eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));

        std::fs::create_dir_all("build").unwrap();
    }

    if args.verbose >= 2 {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_target(false)
            .with_file(false)
            .with_line_number(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .init();

        println!("Lexer Output:");
        println!("Contents:\n{}", contents);
        let lexer = Token::lexer(&contents);
        for token in lexer {
            println!("{:?}", token.unwrap());
        }
    }

    let mut ast = RedoxParser::with_source(&contents).parse().unwrap();
    if args.verbose >= 1 {
        let path = std::path::PathBuf::from("build/main.rxast");
        std::fs::write(path, redox_ast::utils::to_string(&ast)).unwrap();
    }

    let mut type_checker = TypeChecker::new();
    type_checker.type_check(&mut ast).unwrap();

    if args.verbose >= 1 {
        let path = std::path::PathBuf::from("build/main_typed.rxast");
        std::fs::write(path, redox_ast::utils::to_string(&ast)).unwrap();
    }

    let mut ir_generator = IrGenerator::new();
    let module = ir_generator.generate_module(
        ModuleOps {
            name: "main".to_string(),
        },
        ast,
    );

    if args.verbose >= 1 {
        let path = std::path::PathBuf::from("build/main.rxir");
        std::fs::write(path, module.to_string()).unwrap();
    }

    let context = LLVMContext::default();
    let mut codegen = LLVMCodegenBackend::new(&context);

    codegen.gen_module(&module).unwrap();
    codegen
        .write_intermediate(std::path::PathBuf::from("build/main.ll"))
        .unwrap();
    codegen
        .write_object(std::path::PathBuf::from("build/main.o"))
        .unwrap();

    // Then we link with clang
}
