use rust_wasm_runtime::{
    exec::Instance,
    parse::Module,
};
use std::error::Error;
use std::fs;


fn main() -> Result<(), Box<dyn Error>> {
    init();

    // let path = "target/wasm32-wasi/release/rust_wasm_runtime.wasm";
    let path = "example.wasm";
    // let path = "locals.wasm";
    let code = fs::File::open(path)?;
    let module = Module::new(code)?;
    // log::debug!("{:#?}", module);

    let mut instance = Instance::new(module);
    instance.start()?;
    if let Some(mem) = instance.memory() {
        log::info!("Memory {:?}", &mem.data()[0..50]);
    } else {
        log::info!("no memory");
    }


    Ok(())
}

fn init() {
    env_logger::builder()
        .format_timestamp(None)
        .init();
}