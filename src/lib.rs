#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use x86_64::addr::PhysAddr;

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga;

#[macro_export]
macro_rules! c_println {
    () => {
        crate::println!();
        crate::serial_println!();
    };
    ($($arg:tt)*) => {
        crate::println!("{}",format_args!($($arg)*));
        crate::serial_println!("{}",format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! c_print {
    () => {
        crate::print!();
        crate::serial_print!();
    };
    ($($arg:tt)*) => {
        crate::print!("{}",format_args!($($arg)*));
        crate::serial_print!("{}",format_args!($($arg)*));
    };
}

const BANNER: &str = "    ___/-\\___
   |---------|
    | | | | |
    | | | | |
    | | | | |
    | | | | |
    |_______|


";

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info_ptr: usize) {
    println!("{BANNER}");
    println!(" ");
    println!(" ");
    let boot_info = unsafe { multiboot2::load(multiboot_info_ptr).unwrap() };

    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    let kernel_start = PhysAddr::new(
        elf_sections_tag
            .sections()
            .map(|s| s.start_address())
            .min()
            .unwrap(),
    );
    let kernel_end = PhysAddr::new(
        elf_sections_tag
            .sections()
            .map(|s| s.end_address())
            .max()
            .unwrap(),
    );
    let multiboot_start = PhysAddr::new(multiboot_info_ptr.try_into().unwrap());
    let multiboot_end = multiboot_start + boot_info.total_size();

    c_print!("init gdt... ");
    gdt::init();
    c_println!("done");

    c_print!("init interrupts... ");
    interrupts::init();
    c_println!("done");
 
    c_print!("init memory... ");
    memory::init(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        boot_info,
    );
    c_println!("done");

    x86_64::instructions::interrupts::enable();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    c_println!("{info}");
    hlt_loop();
}
