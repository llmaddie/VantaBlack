//! CPUID instruction to the Kernel Module 
#![allow(missing_docs)]
use core::arch::asm;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[inline]
pub(crate) fn cpuid(eax_in: u32, ecx_in: u32) -> CpuidResult {
    let mut eax = eax_in;
    let mut ecx = ecx_in;
    let ebx: u32;
    let edx: u32;

    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_out:e}, ebx",
            "pop rbx",
            ebx_out = lateout(reg) ebx,
            inout("eax") eax,
            inout("ecx") ecx,
            lateout("edx") edx,
        );
    }

    CpuidResult {
        eax,
        ebx,
        ecx,
        edx,
    }
}