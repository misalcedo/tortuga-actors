#![no_main]

// Define the functions that this module will use from the outside world.
// In general, the set of this functions is what we define as an ABI.
// Here we define the "host" namespace for the imports,
// Otherwise it will be "env" by default
#[link(wasm_import_module = "system")]
extern "C" {
    /// Sends a message to the system by passing the memory address of the start of the message.
    fn send(address: *const u8, length: usize);
}

#[no_mangle]
pub unsafe fn receive(address: *const u8, length: usize) {
    send(address, length);
}