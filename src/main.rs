#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
mod vga;

use core::panic::PanicInfo;
/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    for i in 0..100{
        print!("{i}");
    }
    panic!("welp");
    loop {}
}
