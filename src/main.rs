use std::error::Error;
use std::time::Instant;
use rust_wasm_runtime::exec::{Instance, Runtime};
use rust_wasm_runtime::parse::Module;
use rust_wasm_runtime::wasi;

fn main() {
	env_logger::builder().format_timestamp(None).init();

	if let Err(err) = main1() {
		log::error!("{}", err);
	}

}

fn main1() -> Result<(), Box<dyn Error>> {
	// Example code: Prints "Hello" to stdout
	/*let bytecode = [
		1, 12, // Push amount (50)
		4, // Increase memory by 50x u8

		1, 0, // Push addr (0)
		1, 'H' as u8, // Push val ('H')
		5, // Store u8

		1, 1, // Push addr (0)
		1, 'e' as u8, // Push val ('e')
		5, // Store u8

		1, 2, // Push addr (0)
		1, 'l' as u8, // Push val ('l')
		5, // Store u8

		1, 3, // Push addr (0)
		1, 'l' as u8, // Push val ('l')
		5, // Store u8

		1, 4, // Push addr (0)
		1, 'o' as u8, // Push val ('o')
		5, // Store u8


		1, 0, // Push addr (0)
		1, 5, // Push len (5)
		99, // Debug
		6, // Print
	];*/

	let bytecode = [
		0x07, // Function
		'f' as u8, 'o' as u8, 'o' as u8, 0, // Function name

		// Function body

		1, 12, // Push amount (50)
		4, // Increase memory by 50x u8

		1, 0, // Push addr (0)
		1, 'H' as u8, // Push val ('H')
		5, // Store u8

		1, 1, // Push addr (0)
		1, 'e' as u8, // Push val ('e')
		5, // Store u8

		1, 2, // Push addr (0)
		1, 'l' as u8, // Push val ('l')
		5, // Store u8

		1, 3, // Push addr (0)
		1, 'l' as u8, // Push val ('l')
		5, // Store u8

		1, 4, // Push addr (0)
		1, 'o' as u8, // Push val ('o')
		5, // Store u8


		1, 0, // Push addr (0)
		1, 5, // Push len (5)
		99, // Debug


		6, 'p' as u8, 'r' as u8, 'i' as u8, 'n' as u8, 't' as u8, 0, // Call function (print)

		0x08, // Function end
	];

	let module = {
		let start = Instant::now();
		let mut module = Module::new(bytecode)?;
		// Add wasi functions
		wasi::include(&mut module);
		log::info!("Parsing took {:?}", start.elapsed());
		module
	};


	println!("{:#?}", module);

	let mut runtime = Runtime::new();

	let mut instance = Instance {
		module: &module,
		runtime: &mut runtime
	};

	println!("{:#?}", instance);

	{
		let start = Instant::now();
		instance.exec_function("foo")?;
		log::info!("Execution took {:?}", start.elapsed());
	}

	Ok(())
}