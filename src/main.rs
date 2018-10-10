mod token_type;
mod literal;
mod token;
mod scanner;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rox [script]");
        std::process::exit(64);
    }

    let result = if args.len() == 2 {
        let file_name = &args[1];
        run_file(file_name)
    } else {
        run_prompt()
    };

    match result {
        Err(re) => re.report(),
        _ => Ok(()),
    }
}

type RoxResult = Result<(), RoxError>;

fn run_file(path: &String) -> RoxResult {
    // since this program just makes a single, large read of the file
    // it doesn't make sense to bother with a BufReader.
    // maybe this will change in the future.
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    run(contents)
}

fn run_prompt() -> RoxResult {
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush();

        let mut line = String::new();

        line.truncate(0); // read_line appends so we should clear the buffer
        match stdin.read_line(&mut line) {
            Err(_) => println!("Sorry, i didn't catch that!"),
            Ok(_) => run(line).unwrap_or_else(|e| { e.report(); }),
        }
    }
}

fn run(source: String) -> Result<(), RoxError> {
    match scanner::scan(source) {
        Err(e) => Err(RoxError::new(Box::new(e))),
        Ok(tokens) => {
            for token in tokens.iter() {
                println!("{}", token)
            }

            Ok(())
        }
    }
}

struct RoxError {
    error: Box<std::error::Error>,
}

impl RoxError {
    fn new(error: Box<std::error::Error>) -> Self {
        RoxError { error: error }
    }


    fn report(&self) -> std::io::Result<()> {
        let message = if let Some(cause) = self.error.cause() {
            format!("Error: {}\n\t{}", self.error, cause)
        } else {
            format!("Error: {}\n", self.error)
        };

        io::stderr().write_all(message.as_bytes())
    }

}
