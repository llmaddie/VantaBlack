#![no_main]

use core::arch::asm;
use core::sync::atomic::{AtomicPtr, Ordering};
use kernel::alloc::allocator::Kmalloc;
use kernel::prelude::Box;
use kernel::alloc::flags;
// ============================================
// FFI LINUX
// ============================================

mod linux_ffi {
    unsafe extern "C" {
        pub fn __get_free_pages(gfp_mask: u32, order: u32) -> usize;
        pub fn free_pages(addr: usize, order: u32);
        pub fn virt_to_phys(virt: usize) -> u64;
        pub fn printk(fmt: *const u8, ...);
        pub fn smp_processor_id() -> u32;
    }

    pub const GFP_KERNEL: u32 = 0xCC0;
    pub const __GFP_ZERO: u32 = 0x8000;
}

// ============================================
// PRINTK
// ============================================

#[inline]
pub unsafe fn pr_alert(msg: &str) {
    let fmt = b"%s\0".as_ptr();
    let mut buf = [0u8; 256];
    let len = core::cmp::min(msg.len(), 255);
    buf[..len].copy_from_slice(&msg.as_bytes()[..len]);
    linux_ffi::printk(fmt, buf.as_ptr());
}

// ============================================
// CPUID
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

    let ebx_out: u32;
    let edx_out: u32;

    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "mov {tmp:e}, ebx",
            "pop rbx",

            tmp = lateout(reg) ebx_out,

            inout("eax") eax,
            inout("ecx") ecx,

            lateout("edx") edx_out,

            options(preserves_flags),
        );
    }

    CpuidResult {
        eax,
        ebx: ebx_out,
        ecx,
        edx: edx_out,
    }
}

// ============================================
// MSR
// ============================================

#[inline]
pub unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;

    unsafe {
        asm!(
            "rdmsr",

            in("ecx") msr,

            out("eax") low,
            out("edx") high,

            options(nostack, preserves_flags),
        );
    }

    ((high as u64) << 32) | (low as u64)
}

// ============================================
// CR4
// ============================================

#[inline]
pub unsafe fn read_cr4() -> u64 {
    let val: u64;

    unsafe {
        asm!(
            "mov {}, cr4",

            out(reg) val,

            options(nostack),
        );
    }

    val
}

#[inline]
pub unsafe fn write_cr4(val: u64) {
    unsafe {
        asm!(
            "mov cr4, {}",

            in(reg) val,

            options(nostack),
        );
    }
}

// ============================================
// VMXON REGION
// ============================================

pub struct VmxonRegion {
    va: usize,
    pa: u64,
}

impl VmxonRegion {
    pub unsafe fn new() -> Option<Self> {
        let va = unsafe {
            linux_ffi::__get_free_pages(
                linux_ffi::GFP_KERNEL | linux_ffi::__GFP_ZERO,
                0,
            )
        };

        if va == 0 {
            return None;
        }

        let pa = unsafe { linux_ffi::virt_to_phys(va) };

        // VMXON region must be page aligned
        if (pa & 0xFFF) != 0 {
            unsafe {
                linux_ffi::free_pages(va, 0);
            }

            return None;
        }

        // IA32_VMX_BASIC
        let vmx_basic = unsafe { read_msr(0x480) };

        // Write VMCS revision identifier
        unsafe {
            core::ptr::write_volatile(
                va as *mut u32,
                vmx_basic as u32,
            );
        }

        Some(Self { va, pa })
    }

    pub unsafe fn activate(&self) -> Result<(), ()> {
        let mut cf: u8;
        let mut zf: u8;

        let pa_ptr = &self.pa as *const u64;

        unsafe {
            asm!(
                "vmxon [{addr}]",

                "setc {cf}",
                "setz {zf}",

                addr = in(reg) pa_ptr,

                cf = out(reg_byte) cf,
                zf = out(reg_byte) zf,

                options(nostack),
            );
        }

        if cf != 0 || zf != 0 {
            return Err(());
        }

        Ok(())
    }

    pub unsafe fn deactivate(&self) {
        unsafe {
            asm!(
                "vmxoff",
                options(nostack, preserves_flags),
            );
        }
    }

    pub fn va(&self) -> usize {
        self.va
    }

    pub fn pa(&self) -> u64 {
        self.pa
    }
}

// ============================================
// GLOBAL STORAGE
// ============================================

static VMXON_REGION: AtomicPtr<VmxonRegion> =
    AtomicPtr::new(core::ptr::null_mut());

// ============================================
// MODULE INIT
// ============================================

#[no_mangle]
pub extern "C" fn vantablack_init() -> i32 {
    unsafe {
        pr_alert("vantablack_init()\n");

        // ====================================
        // CPUID CHECK
        // ====================================

        let cpu = cpuid(1, 0);

        let vmx_supported = ((cpu.ecx >> 5) & 1) != 0;

        if !vmx_supported {
            pr_alert("VMX unsupported\n");
            return -1;
        }

        pr_alert("VMX supported\n");

        // ====================================
        // IA32_FEATURE_CONTROL
        // ====================================

        let feature_control = read_msr(0x3A);

        let lock_bit = (feature_control & 1) != 0;

        let vmx_enabled = (feature_control & (1 << 2)) != 0;

        if !lock_bit || !vmx_enabled {
            pr_alert("VMX disabled in BIOS\n");
            return -1;
        }

        pr_alert("FEATURE_CONTROL ok\n");

        // ====================================
        // ENABLE CR4.VMXE
        // ====================================

        let mut cr4 = read_cr4();
        cr4 |= 1 << 13;   
        write_cr4(cr4);

        pr_alert("CR4.VMXE enabled\n");

        // ====================================
        // ALLOC VMXON REGION
        // ====================================

        let vmxon = match VmxonRegion::new() {
            Some(v) => v,

            None => {
                pr_alert("VMXON allocation failed\n");
                return -1;
            }
        };

        pr_alert("VMXON region allocated\n");

        // ====================================
        // VMXON
        // ====================================

        if vmxon.activate().is_err() {
            pr_alert("VMXON failed\n");
                linux_ffi::free_pages(vmxon.va(), 0);

            return -1;
        }

        pr_alert("VMXON success\n");

        // ====================================
        // STORE GLOBALLY
        // ====================================

	let boxed = Box::into_raw(
    	Box::<VmxonRegion, Kmalloc>::new(
        	vmxon,
       	 flags::GFP_KERNEL,
    	)
    .unwrap(),
	);
        VMXON_REGION.store(
            boxed,
            Ordering::SeqCst,
        );

        pr_alert("vantablack initialized\n");
    }

    0
}

/// ============================================
// MODULE EXIT
// ============================================

#[no_mangle]
pub extern "C" fn vantablack_exit() {
    unsafe {
        pr_alert("vantablack_exit()\n");

        let ptr = VMXON_REGION.load(Ordering::SeqCst);

        if ptr.is_null() {
            pr_alert("No VMXON region\n");
            return;
        }

        // ====================================
        // VMXOFF 
        // ====================================

        (*ptr).deactivate();
        pr_alert("VMXOFF done\n");

        // ====================================
        // DISABLE CR4.VMXE
        // ====================================

        let mut cr4 = read_cr4();
        cr4 &=1(1 << 13);       
        write_cr4(cr4);
        pr_alert("CR4.VMXE disabled\n");

        // ====================================
        // FREE PAGES
        // ====================================

        linux_ffi::free_pages(
            (*ptr).va(),
            0,
        );
        pr_alert("Pages freed\n");

        // ====================================
        // FREE BOX
        // ====================================

        let _ = Box::from_raw_in(ptr, Kmalloc);
        VMXON_REGION.store(
            core::ptr::null_mut(),
            Ordering::SeqCst,
        );

        pr_alert("vantablack unloaded\n");
    }
}