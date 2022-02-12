use crate::{gdt, println, vga};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::{self, Mutex};
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt[InterruptVectors::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptVectors::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

/// TODO replace with apic
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptVectors {
    Timer = 32,
    Keyboard,
    Serial,
    Mouse = 44,
    Panic = 99,
    Spurious = 255,
}

impl InterruptVectors {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
}

const INDICATOR: [char; 4] = ['\\', '|', '/', '-'];
static INDEX: Mutex<usize> = Mutex::new(0);
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut index = INDEX.lock();
    *index += 1;
    if *index >= INDICATOR.len() {
        *index = 0;
    }

    vga::WRITER.lock().write_at(INDICATOR[*index] as u8, 0, 79);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptVectors::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptVectors::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: INVALID TSS\n{stack_frame:#?}\nerror code: {error_code:?} ({error_code:#b})"
    );
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{stack_frame:#?}\nerror code: {error_code:?} ({error_code:#b})");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{stack_frame:#?}\nerror code: {error_code:?} ({error_code:#b})");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{stack_frame:#?}\nerror code: {error_code:?} ({error_code:#b})");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_address = x86_64::registers::control::Cr2::read();
    panic!("EXCEPTION: PAGE FAULT\n{stack_frame:#?}\naddress: {fault_address:?}\nerror code: {error_code:?} ({error_code:#b})");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// TODO fix kernel Overflow
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{stack_frame:#?}\n(error code {error_code:#b})");
}
