use rust_wasm_runtime::exec::{Module, Runtime};
use std::error::Error;
use std::fs;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let module = parse().expect("Parsing gone wrong");
    let runtime = Runtime::new();
}

fn parse() -> Result<Module, Box<dyn Error>> {
    // let path = "target/wasm32-wasi/release/rust_wasm_runtime.wasm";
    // let path = "example.wasm";
    let path = "locals.wasm";
    let code = fs::File::open(path)?;
    let module = Module::new(code)?;
    println!("{:#?}", module);
    Ok(module)
}

fn exec() {

}