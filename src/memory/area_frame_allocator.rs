use multiboot2::BootInformation;
use x86_64::addr::PhysAddr;
use x86_64::structures::paging::frame::{PhysFrame, PhysFrameRangeInclusive};
use x86_64::structures::paging::{FrameAllocator, Size4KiB};

pub struct AreaFrameAllocator {
    next_free_frame: PhysFrame,
    current_area: Option<PhysFrameRangeInclusive>,
    info: BootInformation,
    kernel: PhysFrameRangeInclusive,
    multiboot: PhysFrameRangeInclusive,
}

impl AreaFrameAllocator {
    pub fn new(
        kernel_start: PhysAddr,
        kernel_end: PhysAddr,
        multiboot_start: PhysAddr,
        multiboot_end: PhysAddr,
        info: BootInformation,
    ) -> AreaFrameAllocator {
        let kernel_start = PhysFrame::containing_address(kernel_start);
        let kernel_end = PhysFrame::containing_address(kernel_end);
        let multiboot_start = PhysFrame::containing_address(multiboot_start);
        let multiboot_end = PhysFrame::containing_address(multiboot_end);
        let mut allocator = AreaFrameAllocator {
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(0)),
            current_area: None,
            info,
            kernel: PhysFrame::range_inclusive(kernel_start, kernel_end),
            multiboot: PhysFrame::range_inclusive(multiboot_start, multiboot_end),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        let next_area = self
            .info
            .memory_map_tag()
            .unwrap()
            .memory_areas()
            .filter(|area| {
                let address = PhysAddr::new(area.start_address() + area.size() - 1);
                PhysFrame::containing_address(address) >= self.next_free_frame
            })
            .min_by_key(|area| area.start_address());
        if let Some(area) = next_area {
            let range = PhysFrame::range_inclusive(
                PhysFrame::containing_address(PhysAddr::new(area.start_address())),
                PhysFrame::containing_address(PhysAddr::new(area.end_address() - 1)),
            );
            let start_frame = range.start;
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }

            self.current_area = Some(range);
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if let Some(area) = self.current_area {
            let frame = self.next_free_frame.clone();
            // the last frame of the current area
            let current_area_last_frame = area.end;
            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame.start_address().as_u64() < 1024 * 1024 {
                // We are in the scary memory below 1MiB
                self.next_free_frame = PhysFrame::containing_address(PhysAddr::new(1024 * 1024));
            } else if frame >= self.kernel.start && frame <= self.kernel.end {
                // `frame` is used by the kernel
                self.next_free_frame = PhysFrame::from_start_address(
                    self.kernel.end.start_address() + self.kernel.end.size(),
                )
                .unwrap();
            } else if frame >= self.multiboot.start && frame <= self.multiboot.end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = PhysFrame::from_start_address(
                    self.multiboot.end.start_address() + self.multiboot.end.size(),
                )
                .unwrap();
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame = PhysFrame::from_start_address(
                    self.next_free_frame.start_address() + self.next_free_frame.size(),
                )
                .unwrap();
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }
}
