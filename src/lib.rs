#![no_std]
#![no_main]

use core::panic::PanicInfo;

pub mod serial;
pub mod vga;


#[no_mangle]
pub extern fn rust_main() {
    println!("Hello World");
    serial_println!("wohoo!!! {}", 1);



    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    println!("{}", info);
    loop {}
}

