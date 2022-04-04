use rust_wasm_runtime::{
    exec::Instance,
    parse::Module,
};
use std::error::Error;
use std::fs;
use tracing::{info, Level};

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
        info!("Memory {:?}", &mem.data()[0..50]);
    } else {
        info!("no memory");
    }


    Ok(())
}

fn init() {
    use tracing_subscriber::fmt::format::FmtSpan;

    tracing_subscriber::fmt::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(Level::TRACE)
        .with_target(false)
        .without_time()
        .with_span_events(FmtSpan::ENTER)
        .init();
}