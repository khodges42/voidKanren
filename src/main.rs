mod frontend;
mod runtime;
mod ui;

mod kanren;
mod kanren_forms;

use std::{env as std_env, fs};

fn main() {
    let args: Vec<String> = std_env::args().collect();

    if args.len() > 1 {
        let path = &args[1];

        let source = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {path}: {err}"));

        ui::repl::run_source(&source);
    } else {
        ui::repl::run();
    }
}