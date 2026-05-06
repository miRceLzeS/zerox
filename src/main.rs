use std::{io::Write, process::ExitCode};

use zerox::{
    Result,
    error::{Error, ErrorKind},
};

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

        _ => Err(Error::new(ErrorKind::CLIErrorr(
            "Usage: zerox [file]".into(),
        ))),
    }
}

fn run_prompt() -> Result<()> {
    let mut input = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        std::io::stdin().read_line(&mut input)?;

        interpret(&input)?;
        input.clear();
    }
}

fn run_file(file_name: &str) -> Result<()> {
    let content = std::fs::read_to_string(file_name)?;
    interpret(&content)
}

fn interpret(source_code: &str) -> Result<()> {
    println!("{source_code}");

    Ok(())
}
