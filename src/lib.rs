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
macro_rules! cprintln {
    () => {
        crate::println!();
        crate::sprintln!();
    };
    ($($arg:tt)*) => {
        crate::println!("{}",format_args!($($arg)*));
        crate::sprintln!("{}",format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! cprint {
    () => {
        crate::print!();
        crate::sprint!();
    };
    ($($arg:tt)*) => {
        crate::print!("{}",format_args!($($arg)*));
        crate::sprint!("{}",format_args!($($arg)*));
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

    cprint!("init gdt... ");
    gdt::init();
    cprintln!("done");

    cprint!("init interrupts... ");
    interrupts::init();
    cprintln!("done");
 
    cprint!("init memory... ");
    memory::init(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        boot_info,
    );
    cprintln!("done");

    x86_64::instructions::interrupts::enable();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cprintln!("{info}");
    hlt_loop();
}
