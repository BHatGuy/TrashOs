#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(trashos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use trashos::{println, serial_println};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    trashos::test_panic_handler(info)
}
