// ============================================
// INSTRUCTIONS VMX BRUTES + MSR BASIQUES
// Aucune allocation mémoire, aucune logique métier
// ============================================

use core::arch::asm;

/// VMXON - Active VMX operation
#[inline(always)]
pub unsafe fn vmxon(phys_addr: u64) -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!(
        "vmxon [{0}]",
        in(reg) &phys_addr,
        out("rflags") rflags,
        options(nostack, preserves_flags)
    );
    check_vmx_error(rflags)
}

/// VMXOFF - Désactive VMX
#[inline(always)]
pub unsafe fn vmxoff() {
    asm!("vmxoff", options(nostack, preserves_flags));
}

/// VMCLEAR
#[inline(always)]
pub unsafe fn vmclear(phys_addr: u64) -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("vmclear [{0}]", in(reg) &phys_addr, out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

/// VMPTRLD
#[inline(always)]
pub unsafe fn vmptrld(phys_addr: u64) -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("vmptrld [{0}]", in(reg) &phys_addr, out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

/// VMWRITE
#[inline(always)]
pub unsafe fn vmwrite(field: u64, value: u64) -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("vmwrite {1}, {0}", in(reg) field, in(reg) value, out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

/// VMREAD
#[inline(always)]
pub unsafe fn vmread(field: u64) -> Result<u64, VmxInstructionError> {
    let mut value: u64;
    let mut rflags: u64;
    asm!("vmread {1}, {0}", out(reg) value, in(reg) field, out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)?;
    Ok(value)
}

/// VMLAUNCH
#[inline(always)]
pub unsafe fn vmlaunch() -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("vmlaunch", out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

/// VMRESUME
#[inline(always)]
pub unsafe fn vmresume() -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("vmresume", out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

/// INVEPT - Invalidate EPT Entries
#[inline(always)]
pub unsafe fn invept(inv_type: u64, descriptor: &InveptDescriptor) -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("invept {1}, [{0}]", in(reg) descriptor, in(reg) inv_type, out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

/// INVVPID - Invalidate VPID
#[inline(always)]
pub unsafe fn invvpid(inv_type: u64, descriptor: &InvvpidDescriptor) -> Result<(), VmxInstructionError> {
    let mut rflags: u64;
    asm!("invvpid {1}, [{0}]", in(reg) descriptor, in(reg) inv_type, out("rflags") rflags, options(nostack, preserves_flags));
    check_vmx_error(rflags)
}

// ==== MSR BASIQUES (utilisés par intel.rs lui-même) ====

/// Read MSR
#[inline(always)]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nostack, preserves_flags));
    ((high as u64) << 32) | (low as u64)
}

/// Write MSR
#[inline(always)]
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high, options(nostack, preserves_flags));
}

// ==== Types et helpers ====

#[derive(Debug, Clone, Copy)]
pub enum VmxInstructionError {
    VmFailValid,   // VMfailValid - error code dans VM-instruction error field
    VmFailInvalid, // VMfailInvalid - VMCS pointer invalid
}

#[repr(C)]
pub struct InveptDescriptor {
    pub eptp: u64,  
    pub reserved: u64,
}

#[repr(C)]
pub struct InvvpidDescriptor {
    pub vpid: u64,
    pub linear_address: u64,
}

fn check_vmx_error(rflags: u64) -> Result<(), VmxInstructionError> {
    if (rflags & 0x1) != 0 {      // CF=1
        Err(VmxInstructionError::VmFailInvalid)
    } else if (rflags & 0x40) != 0 { // ZF=1
        Err(VmxInstructionError::VmFailValid)
    } else {
        Ok(())
    }
}
