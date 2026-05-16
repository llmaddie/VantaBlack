#![no_std]

use core::arch::asm;


#[derive(Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32;
    pub ebx: u32;
    pub ecx: u32;
    pub edx: u32;
}

#[inline]
pub fn cpuid(eax_in: u32, exc_in: u32) -> CpuidResult {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;

    unsafe {
        asm!(
            "cpuid"
            inlateout("eax") eax_in => eax,
            lateout("ebx") ebx,
            inlateout("ecx") ecx_in => ecx,
            lateout("edx") edx,
            options(nomem, nostack),
        );
    }
    CpuidResult { eax, ebx, ecx, edx }
}