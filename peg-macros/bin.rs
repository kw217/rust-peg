//! Standalone version of rust-peg used for bootstrapping the meta-grammar

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;

use std::env;
use std::fs::File;
use std::io::{stderr, stdin, stdout};
use std::io::{Read, Write};
use std::path::Path;
use std::process;

// This can't use the `peg` crate as it would be a circular dependency, but the generated code in grammar.rs
// requires `::peg` paths.
extern crate peg_runtime as peg;

mod analysis;
mod ast;
mod grammar;
mod tokens;
mod translate;

fn main() {
    let args = env::args_os().collect::<Vec<_>>();
    let progname = &args[0];
    let mut log = stderr();

    let mut source = String::new();

    if args.len() == 2 && &args[1] != "-h" {
        File::open(Path::new(&args[1]))
            .unwrap()
            .read_to_string(&mut source)
            .unwrap();
    } else if args.len() == 1 {
        stdin().read_to_string(&mut source).unwrap();
    } else {
        writeln!(log, "Usage: {} [file]", progname.to_string_lossy()).unwrap();
        process::exit(0);
    }

    let source_tokens = source.parse().expect("Error tokenizing input");
    let input_tokens = tokens::FlatTokenStream::new(source_tokens);
    let grammar = match grammar::peg::peg_grammar(&input_tokens).into_result() {
        Ok(g) => g,
        Err(err) => {
            eprintln!("Failed to parse grammar: expected {}", err.expected);
            process::exit(1);
        }
    };
    let parser_tokens = translate::compile_grammar(&grammar);
    let mut out = stdout();
    writeln!(&mut out, "// Generated by rust-peg. Do not edit.").unwrap();
    write!(&mut out, "{}", parser_tokens).unwrap();
}
