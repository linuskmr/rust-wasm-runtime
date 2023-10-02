use rust_wasm_runtime::{
    exec::Instance,
    parse::Module,
};
use std::error::Error;
use std::fs;


fn main() -> Result<(), Box<dyn Error>> {
    init_logger();

    // let path = "target/wasm32-wasi/release/rust_wasm_runtime.wasm";
    let path = "example.wasm";
    // let path = "locals.wasm";
    let code = fs::File::open(path)?;
    let module = Module::new(code)?;
    tracing::debug!("{:#?}", module);

    let mut instance = Instance::new(module);
    instance.start()?;
    if let Some(mem) = instance.memory() {
        tracing::info!("Memory dump: {:?}", &mem.data()[0..50]);
    } else {
        tracing::info!("no memory");
    }


    Ok(())
}

fn init_logger() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::Registry::default()
        .with(
            tracing_tree::HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        ).init();
}