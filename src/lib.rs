#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga;


pub fn init() {
    print!("init gdt... ");
    gdt::init();
    println!("done");

    print!("init interrupts... ");
    interrupts::init();
    println!("done");
}

#[no_mangle]
pub extern "C" fn rust_main() {
    init();
    // println!("{BANNER}");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    println!("{}", info);
    loop {}
}
