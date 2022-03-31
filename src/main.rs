use rust_wasm_runtime::parse::Module;
use std::error::Error;
use std::fs;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if let Err(err) = main_() {
        log::error!("{}", err);
    }
}

fn main_() -> Result<(), Box<dyn Error>> {
    // let path = "target/wasm32-wasi/release/rust_wasm_runtime.wasm";
    let path = "example.wasm";
    let code = fs::File::open(path)?;
    let module = Module::new(code)?;
    println!("{:#?}", module);
    Ok(())
}
