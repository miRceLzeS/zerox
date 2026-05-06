use std::{io::Write, process::ExitCode};

use zerox::{Error, Result};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match run(&args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprint!("{}", err);
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Vec<String>) -> Result<()> {
    match args.len() {
        0 => run_prompt(),

        1 => run_file(&args[0]),

        _ => Err(Error::CLIErrorr(format!("Usage: zerox [file]"))),
    }
}

fn run_prompt() -> Result<()> {
    let mut input = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let n = std::io::stdin().read_line(&mut input)?;
        if n == 0 {
            println!("");
            break;
        }

        if let Err(err) = interpret(&input) {
            println!("{}", err);
        }
        input.clear();
    }

    Ok(())
}

fn run_file(file_name: &str) -> Result<()> {
    let content = std::fs::read_to_string(file_name)?;
    interpret(&content)
}

fn interpret(source_code: &str) -> Result<()> {
    let mut l = zerox::Lexer::new(source_code);
    let tokens = l.lex().unwrap();
    println!("{:?}", tokens);

    Ok(())
}
