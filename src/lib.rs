#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;
use task::{executor::Executor, Task, keyboard};
use x86_64::addr::PhysAddr;
extern crate alloc;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga;
pub mod task;

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

fn init(multiboot_info_ptr: usize) {
    let boot_info =
        unsafe { multiboot2::load(multiboot_info_ptr).expect("Multiboot not present!") };
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

    cprint!("init gdt ");
    gdt::init();
    cprintln!("[done]");

    cprint!("init interrupts ");
    interrupts::init();
    cprintln!("[done]");

    cprint!("init memory ");
    memory::init(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        boot_info,
    );
    cprintln!("[done]");

    x86_64::instructions::interrupts::enable();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info_ptr: usize) {
    init(multiboot_info_ptr);

    println!(" ");
    println!(" ");
    println!("{BANNER}");
    println!(" ");
    println!(" ");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    cprintln!("{info}");
    hlt_loop();
}
