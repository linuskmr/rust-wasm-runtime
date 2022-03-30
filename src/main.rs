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
    let code = fs::File::open("example.wasm")?;
    let module = Module::new(code)?;
    println!("{:#?}", module);
    Ok(())
}
