#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;
use core::sync::atomic::{AtomicPtr, Ordering};

// ============================================
// FFI LINUX - Bindings corrigés
// ============================================

mod linux_ffi {
    extern "C" {
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
// PRINTK - Fonctions wrapper (pas de macro cassée)
// ============================================

#[inline]
pub unsafe fn pr_alert(msg: &str) {
    // printk attend un null-terminated C string
    // On utilise une approche simple : buffer statique temporaire
    // NOTE: en production, utiliser une meilleure stratégie
    let bytes = msg.as_bytes();
    let mut buf = [0u8; 256];
    let len = core::cmp::min(bytes.len(), 255);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len] = 0;
    linux_ffi::printk(buf.as_ptr());
}

// ============================================
// CPUID + MSR - Corrigés
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
    let mut ebx_out: u32;
    let mut edx_out: u32;

    asm!(
        "push rbx",           // Sauvegarde rbx
        "cpuid",
        "mov {tmp:e}, ebx",   // Copie ebx dans tmp
        "pop rbx",            // Restauration rbx
        tmp = lateout(reg) ebx_out,
        inout("eax") eax,
        inout("ecx") ecx,
        lateout("edx") edx_out,
        options(nostack, preserves_flags),
    );

    CpuidResult {
        eax,
        ebx: ebx_out,
        ecx,
        edx: edx_out,
    }
}

#[inline]
pub unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nostack, preserves_flags),
    );
    ((high as u64) << 32) | (low as u64)
}

#[inline]
pub unsafe fn read_cr4() -> u64 {
    let val: u64;
    asm!(
        "mov {}, cr4",
        out(reg) val,
        options(nostack),
        // NOTE: pas de preserves_flags car mov cr4 peut affecter certains flags
    );
    val
}

#[inline]
pub unsafe fn write_cr4(val: u64) {
    asm!(
        "mov cr4, {}",
        in(reg) val,
        options(nostack),
    );
}

// ============================================
// VMXON / VMCS - Corrigé
// ============================================

pub struct VmxonRegion {
    va: usize,
    pa: u64,
}

impl VmxonRegion {
    pub unsafe fn new() -> Option<Self> {
        let va = linux_ffi::__get_free_pages(
            linux_ffi::GFP_KERNEL | linux_ffi::__GFP_ZERO,
            0
        );
        if va == 0 {
            return None;
        }

        let pa = linux_ffi::virt_to_phys(va);
        if (pa & 0xFFF) != 0 {
            linux_ffi::free_pages(va, 0);
            return None;
        }

        // Écriture du VMX revision ID
        let vmx_basic = read_msr(0x480);
        core::ptr::write_volatile(va as *mut u32, vmx_basic as u32);

        Some(Self { va, pa })
    }

    pub unsafe fn activate(&self) -> Result<(), ()> {
        let mut cf: u8;
        let mut zf: u8;

        asm!(
            "vmxon [{0}]",
            "setc {1}",      // CF = 1 si erreur
            "setz {2}",      // ZF = 1 si VMXON déjà actif
            in(reg) &self.pa,
            out(reg_byte) cf,
            out(reg_byte) zf,
            options(nostack),
        );

        if cf != 0 || zf != 0 {
            return Err(());
        }
        Ok(())
    }

    pub unsafe fn deactivate(&self) {
        asm!("vmxoff", options(nostack, preserves_flags));
    }

    pub fn get_va(&self) -> usize {
        self.va
    }

    pub fn get_pa(&self) -> u64 {
        self.pa
    }
}

impl Drop for VmxonRegion {
    fn drop(&mut self) {
        unsafe {
            // NOTE: vmxoff doit être appelé AVANT free_pages
            // C'est la responsabilité de l'appelant de le faire
            linux_ffi::free_pages(self.va, 0);
        }
    }
}

// ============================================
// STOCKAGE GLOBAL - 
// ============================================

static VMXON_REGION: AtomicPtr<VmxonRegion> = AtomicPtr::new(core::ptr::null_mut());

// ============================================
// MODULE INIT / EXIT - 
// ============================================

#[no_mangle]
pub extern "C" fn vantablack_init() -> i32 {
    unsafe {
        // 1. Vérification CPUID
        let cpu = cpuid(1, 0);
        let vmx_bit = (cpu.ecx >> 5) & 1;
        if vmx_bit == 0 {
            pr_alert("VMX not available\\n");
            return -1;
        }

        // 2. Vérification MSR IA32_FEATURE_CONTROL
        let msr = read_msr(0x3A);
        if (msr & 1) == 0 || (msr & (1 << 2)) == 0 {
            pr_alert("VMX disabled by BIOS\\n");
            return -1;
        }

        pr_alert("CPUID/MSR checks passed\\n");

        // 3. Vérification et activation CR4.VMXE
        let mut cr4 = read_cr4();
        cr4 |= 1 << 13; // VMXE
        write_cr4(cr4);

        // 4. Allocation et activation VMXON
        let vmxon = match VmxonRegion::new() {
            Some(v) => v,
            None => {
                pr_alert("VMXON allocation failed\\n");
                return -1;
            }
        };

        if let Err(_) = vmxon.activate() {
            pr_alert("VMXON failed\\n");
            return -1;
        }

        // 5. Stockage global pour éviter le drop
        let boxed = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(vmxon));
        VMXON_REGION.store(boxed, Ordering::SeqCst);

        pr_alert("VMXON SUCCESS!\\n");
    }

    0
}

#[no_mangle]
pub extern "C" fn vantablack_exit() {
    unsafe {
        let ptr = VMXON_REGION.load(Ordering::SeqCst);
        if !ptr.is_null() {
            // 1. Désactiver VMX avant de libérer la mémoire
            (*ptr).deactivate();
            pr_alert("VMXOFF\\n");

            // 2. Libérer la mémoire
            let _ = alloc::boxed::Box::from_raw(ptr);
            VMXON_REGION.store(core::ptr::null_mut(), Ordering::SeqCst);
        }
    }
}

// ============================================
// PANIC HANDLER
// ============================================

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}
