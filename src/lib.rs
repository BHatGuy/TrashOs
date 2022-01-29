#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga;

const BANNER: &str = "    ___/-\\___
   |---------|
    | | | | |
    | | | | |
    | | | | |
    | | | | |
    |_______|


";

pub fn init() {
    serial_print!("init gdt... ");
    gdt::init();
    serial_println!("done");

    serial_print!("init interrupts... ");
    interrupts::init();
    serial_println!("done");
}

#[no_mangle]
pub extern "C" fn rust_main() {
    init();
    println!("{BANNER}");
    println!(" ");
    println!(" ");
    for i in "\\|/-".chars().cycle() {
        print!("\r{i}");
    }
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    println!("{}", info);
    loop {}
}
