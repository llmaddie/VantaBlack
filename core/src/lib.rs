#![no_std]
#![no_main]

use kernel::prelude::*;
use kernel::module;
use core::arch::asm;

module! {
    type: Vantablack,
    name: "vantablack",
    authors: ["maddie"],
    description: "Type-2 Hypervisor in Rust",
    license: "GPL",
}

struct Vantablack;

impl kernel::Module for Vantablack {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Vantablack: Loading hypervisor...\n");
        
        unsafe {
            // 1. Check VMX support
            if !Self::check_vmx_support() {
                pr_alert!("Vantablack: VMX not supported\n");
                return Err(kernel::error::code::ENODEV);
            }

            // 2. Enable CR4.VMXE
            let mut cr4 = Self::read_cr4();
            cr4 |= 1 << 13;
            Self::write_cr4(cr4);

            // 3. Allocate VMXON region
            let vmxon_pa = Self::alloc_vmxon_region()?;
            
            // 4. VMXON !
            let mut rflags: u64;
            asm!(
                "vmxon [{0}]",
                in(reg) &vmxon_pa,
                out("rflags") rflags,
                options(nostack, preserves_flags)
            );
            
            if (rflags & 0x1) != 0 || (rflags & 0x40) != 0 {
                pr_alert!("Vantablack: VMXON failed\n");
                return Err(kernel::error::code::ENODEV);
            }

            pr_alert!("Vantablack: VMXON success! PA={:#x}\n", vmxon_pa);
        }

        Ok(Vantablack)
    }
}

impl Drop for Vantablack {
    fn drop(&mut self) {
        unsafe {
            asm!("vmxoff", options(nostack, preserves_flags));
            pr_info!("Vantablack: VMXOFF\n");
        }
    }
}

impl Vantablack {
    unsafe fn check_vmx_support() -> bool {
        // CPUID leaf 1, ECX bit 5
        let mut eax = 1u32;
        let mut ecx = 0u32;
        let _ebx: u32;
        let _edx: u32;
        
        asm!(
            "cpuid",
            inout("eax") eax,
            lateout("ebx") _ebx,
            inout("ecx") ecx,
            lateout("edx") _edx,
        );
        
        let vmx_bit = (ecx >> 5) & 1;
        if vmx_bit == 0 {
            return false;
        }

        // MSR 0x3A check
        let msr = Self::read_msr(0x3A);
        if (msr & 1) == 0 || (msr & (1 << 2)) == 0 {
            return false;
        }
        
        true
    }

    unsafe fn read_msr(msr: u32) -> u64 {
        let low: u32;
        let high: u32;
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
        ((high as u64) << 32) | (low as u64)
    }

    unsafe fn read_cr4() -> u64 {
        let val: u64;
        asm!("mov {}, cr4", out(reg) val, options(nostack, preserves_flags));
        val
    }

    unsafe fn write_cr4(val: u64) {
        asm!("mov cr4, {}", in(reg) val, options(nostack, preserves_flags));
    }

    unsafe fn alloc_vmxon_region() -> Result<u64, Error> {
        // Allocation via kernel allocator
        let layout = Layout::from_size_align(4096, 4096)
            .map_err(|_| kernel::error::code::ENOMEM)?;
        
        let ptr = alloc::alloc::alloc(layout);
        if ptr.is_null() {
            return Err(kernel::error::code::ENOMEM);
        }

        // Zero the page
        core::ptr::write_bytes(ptr, 0, 4096);

        // Get physical address - Rust for Linux expose virt_to_phys
        let pa = kernel::mm::virt_to_phys(ptr as usize);
        
        // Write VMX revision ID
        let vmx_basic = Self::read_msr(0x480);
        core::ptr::write_volatile(ptr as *mut u32, vmx_basic as u32);

        pr_alert!("Vantablack: VMXON region PA={:#x}, rev={:#x}\n", pa, vmx_basic as u32);
        
        Ok(pa as u64)
    }
}