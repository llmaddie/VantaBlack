// Tout ce qui vient du kernel C

extern "C" {
    pub fn __get_free_pages(gfp_mask: u32, order: u32) -> *mut u8;
    pub fn free_pages(addr: *mut u8, order: u32);
    pub fn virt_to_phys(virt: *mut u8) -> u64;
    pub fn printk(fmt: *const u8, ...);
}

pub const GFP_KERNEL: u32 = 0xCC0;
pub const __GFP_ZERO: u32 = 0x8000;

pub unsafe fn alloc_zeroed_page() -> *mut u8 {
    __get_free_pages(GFP_KERNEL | __GFP_ZERO, 0)
}

pub unsafe fn free_page(addr: *mut u8) {
    free_pages(addr, 0);
}