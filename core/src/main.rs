#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

// ============================================
// FFI LINUX - Bindings minimaux
// ============================================

mod linux_ffi {
   // use core::ffi::c_void; unused so for debug i comment them 
    
    extern "C" {
        pub fn __get_free_pages(gfp_mask: u32, order: u32) -> *mut u8;
        pub fn free_pages(addr: *mut u8, order: u32);
        pub fn virt_to_phys(virt: *mut u8) -> u64;
        pub fn printk(fmt: *const u8, ...);
        pub fn smp_processor_id() -> u32;
    }
    
    pub const GFP_KERNEL: u32 = 0xCC0;
    pub const __GFP_ZERO: u32 = 0x8000;
}

// ============================================
// MACROS PRINTK
// ============================================

#[macro_export]
macro_rules! pr_info {
    ($($arg:tt)*) => {
        unsafe {
            let msg = concat!($($arg)*, "\0");
            $crate::linux_ffi::printk(msg.as_ptr());
        }
    };
}

#[macro_export]
macro_rules! pr_alert {
    ($($arg:tt)*) => {
        unsafe {
            let msg = concat!($($arg)*, "\0");
            $crate::linux_ffi::printk(msg.as_ptr());
        }
    };
}

// ============================================
// CPUID + MSR
// ============================================

#[derive(Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[inline]
pub unsafe fn cpuid(eax_in: u32, ecx_in: u32) -> CpuidResult {
    let mut eax = eax_in;
    let mut ecx = ecx_in;
    let ebx: u32;
    let edx: u32;
unsafe{
    asm!(
        "mov {tmp:r}, rbx",
        "cpuid",
        "xchg {tmp:r}, rbx",

        tmp = lateout(reg) ebx,
        inout("eax") eax,
        inout("ecx") ecx,
        lateout("edx") edx,

        options(nostack),
    );
}
#[inline]
pub unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe{
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
    );
}
    ((high as u64) << 32) | (low as u64)
}

#[inline]
pub unsafe fn read_cr4() -> u64 {
    let val: u64; 
    unsafe{
    asm!("mov {}, cr4", out(reg) val, options(nostack, preserves_flags));
    }
    val
}

#[inline]
pub unsafe fn write_cr4(val: u64) {
    asm!("mov cr4, {}", in(reg) val, options(nostack, preserves_flags));
}

// ============================================
// VMXON / VMCS
// ============================================

pub struct VmxonRegion {
    va: *mut u8,
    pa: u64,
}

impl VmxonRegion {
    pub unsafe fn new() -> Option<Self> {
        let va = linux_ffi::__get_free_pages(
            linux_ffi::GFP_KERNEL | linux_ffi::__GFP_ZERO, 
            0
        );
        if va.is_null() {
            return None;
        }

        let pa = linux_ffi::virt_to_phys(va);
        if (pa & 0xFFF) != 0 {
            linux_ffi::free_pages(va, 0);
            return None;
        }

        // Write VMX revision ID
        let vmx_basic = read_msr(0x480);
        core::ptr::write_volatile(va as *mut u32, vmx_basic as u32);

        pr_alert!("VMXON region: PA={:#x}, rev={:#x}\n", pa, vmx_basic as u32);

        Some(Self { va, pa })
    }

    pub unsafe fn activate(&self) -> Result<(), ()> {
        let mut rflags: u64;
        asm!(
            "vmxon [{0}]",
            in(reg) &self.pa,
            out("rflags") rflags,
            options(nostack, preserves_flags)
        );

        if (rflags & 0x1) != 0 || (rflags & 0x40) != 0 {
            return Err(());
        }
        Ok(())
    }
}

impl Drop for VmxonRegion {
    fn drop(&mut self) {
        unsafe { linux_ffi::free_pages(self.va, 0); }
    }
}

// ============================================
// MODULE INIT / EXIT
// ============================================

#[no_mangle]
pub extern "C" fn vantablack_init() -> i32 {
    unsafe {
        // 1. CPUID check
        let cpu = cpuid(1, 0);
        let vmx_bit = (cpu.ecx >> 5) & 1;
        if vmx_bit == 0 {
            pr_alert!("VMX not available\n");
            return -1;
        }

        // 2. MSR check
        let msr = read_msr(0x3A);
        if (msr & 1) == 0 || (msr & (1 << 2)) == 0 {
            pr_alert!("VMX disabled by BIOS\n");
            return -1;
        }

        pr_info!("CPUID/MSR checks passed\n");

        // 3. Enable VMX
        let mut cr4 = read_cr4();
        cr4 |= 1 << 13; // VMXE
        write_cr4(cr4);

        // 4. VMXON
        let vmxon = match VmxonRegion::new() {
            Some(v) => v,
            None => {
                pr_alert!("VMXON allocation failed\n");
                return -1;
            }
        };

        if let Err(_) = vmxon.activate() {
            pr_alert!("VMXON failed\n");
            return -1;
        }

        pr_alert!("VMXON SUCCESS!\n");
    }

    0
}

#[no_mangle]
pub extern "C" fn vantablack_exit() {
    unsafe {
        asm!("vmxoff", options(nostack, preserves_flags));
        pr_info!("VMXOFF\n");
    }
}

// ============================================
// PANIC HANDLER
// ============================================

#[panic_handler]
fn panic(_info: &PanicInfo<'_) -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)); }
    }
    }
