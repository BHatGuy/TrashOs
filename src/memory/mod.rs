use multiboot2::BootInformation;
use x86_64::addr::PhysAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    frame::PhysFrame, mapper::PageTableFrameMapping, FrameAllocator, MappedPageTable, Mapper, Page,
    PageTable, Size1GiB, Size4KiB,
};
use x86_64::structures::paging::{PageSize, PageTableFlags as Flags, OffsetPageTable};
use x86_64::VirtAddr;

mod area_frame_allocator;
pub mod allocator;

struct StartMapping {}

unsafe impl PageTableFrameMapping for StartMapping {
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

fn create_total_offset_mapping(
    offset: VirtAddr,
    end: PhysAddr,
    allocator: &mut dyn FrameAllocator<Size4KiB>,
    level_4_table: &mut PageTable,
) {
    let mut mapper = unsafe { MappedPageTable::new(level_4_table, StartMapping {}) };
    let mut frame_addr = PhysAddr::new(0);
    while frame_addr < end {
        let page: Page<Size1GiB> = Page::containing_address(offset + frame_addr.as_u64());
        let frame = PhysFrame::containing_address(frame_addr);

        unsafe {
            mapper
                .map_to(page, frame, Flags::WRITABLE | Flags::PRESENT, allocator)
                .unwrap()
                .ignore();
        }
        frame_addr += Size1GiB::SIZE;
    }
}

pub fn init(
    kernel_start: PhysAddr,
    kernel_end: PhysAddr,
    multiboot_start: PhysAddr,
    multiboot_end: PhysAddr,
    boot_info: BootInformation,
) {
    // TODO: use better allocator

    let tag = boot_info.memory_map_tag().unwrap();
    let end = PhysAddr::new(
        tag.all_memory_areas()
            .map(|area| area.start_address() + area.size())
            .max()
            .unwrap(),
    );

    let mut allocator = area_frame_allocator::AreaFrameAllocator::new(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        boot_info,
    );

    let level_4_table = unsafe { active_level_4_table(VirtAddr::new(0)) };

    let offset = VirtAddr::new(1 << 44); // 16 TiB
    create_total_offset_mapping(offset, end, &mut allocator, level_4_table);

    let mut mapper = unsafe{OffsetPageTable::new(level_4_table, offset)};
    allocator::init_heap(&mut mapper, &mut allocator).expect("heap initialization failed");
}
