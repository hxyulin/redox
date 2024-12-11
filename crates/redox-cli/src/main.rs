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
    #[clap(short, long, default_value = "false")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let mut contents = String::new();
    args.input
        .open()
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    if args.verbose {
        println!("Contents:\n{}", contents);

        let lexer = Token::lexer(&contents);
        for token in lexer {
            println!("{:?}", token);
        }
    }

    let mut ast = RedoxParser::with_source(&contents).parse().unwrap();
    if args.verbose {
        println!("AST:\n{:#?}", ast);
    }

    let mut type_checker = TypeChecker::new();
    type_checker.type_check(&mut ast).unwrap();

    if args.verbose {
        println!("Type Checked AST:\n{:#?}", ast);
    }

    let mut ir_generator = IrGenerator::new();
    let module = ir_generator.generate_module(
        ModuleOps {
            name: "main".to_string(),
        },
        ast,
    );

    // TODO: Ir String representation is not implemented
    dbg!(&module);

    let context = LLVMContext::default();
    let mut codegen = LLVMCodegenBackend::new(&context);

    codegen.gen_module(&module).unwrap();
    codegen
        .write_intermediate(std::path::PathBuf::from("main.ll"))
        .unwrap();
    codegen
        .write_object(std::path::PathBuf::from("main.o"))
        .unwrap();

    // Then we link with clang
}
