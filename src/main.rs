use std::{
    env,
    fs,
    io::prelude::*,
};
use rustyline::Editor;
use atto::{
    parse,
    exec,
};

fn prompt() {
    println!("Welcome to the Atto prompt.");

    let mut rl = Editor::<()>::new();
    while let Ok(line) = rl.readline(">> ") {
        rl.add_history_entry(line.as_ref());

        let _ = exec::ast::exec(&line)
            .map_err(|err| print!("{:?}", err));
    }
}

fn exec(fname: &str) {
    let mut code = String::new();
    match fs::File::open(fname) {
        Ok(mut file) => { file.read_to_string(&mut code).unwrap(); },
        Err(_) => println!("Could not open file '{}'", fname),
    }

    exec::ast::exec(&code)
        .map_err(|err| print!("{:?}", err));
}

fn usage() {
    println!("Usage: atto [file]");
}

fn main() {
    match &env::args().nth(1) {
        None => prompt(),
        Some(arg) if env::args().count() == 2 => exec(arg),
        Some(_) => usage(),
    }
}
