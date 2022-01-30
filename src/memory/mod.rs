use multiboot2::BootInformation;
use x86_64::addr::PhysAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTableFlags as Flags;
use x86_64::structures::paging::{
    frame::PhysFrame, mapper::PageTableFrameMapping, MappedPageTable, Mapper, Page, PageTable,
    Size4KiB,
};

use x86_64::VirtAddr;

mod area_frame_allocator;

struct StartMapper {}

unsafe impl PageTableFrameMapping for StartMapper {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        frame.start_address().as_u64() as *mut PageTable
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub fn init(
    kernel_start: PhysAddr,
    kernel_end: PhysAddr,
    multiboot_start: PhysAddr,
    multiboot_end: PhysAddr,
    memory_areas: BootInformation,
) {
    let mut allocator = area_frame_allocator::AreaFrameAllocator::new(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        memory_areas,
    );
    unsafe {
        let level_4_table = active_level_4_table(VirtAddr::new(0));

        let mut mapper = MappedPageTable::new(level_4_table, StartMapper {});
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0xfec00000));
        let frame = PhysFrame::containing_address(PhysAddr::new(0xfec00000));
        mapper
            .map_to(
                page,
                frame,
                Flags::PRESENT | Flags::WRITABLE,
                &mut allocator,
            )
            .unwrap()
            .ignore();
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0xfee00320));
        let frame = PhysFrame::containing_address(PhysAddr::new(0xfee00320));
        mapper
            .map_to(
                page,
                frame,
                Flags::PRESENT | Flags::WRITABLE,
                &mut allocator,
            )
            .unwrap()
            .ignore();
    };
}
