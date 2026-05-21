// src/cpu.rs
use crate::intel::{rdmsr, wrmsr, read_cr4, write_cr4};

const MSR_IA32_FEATURE_CONTROL: u32 = 0x3A;
const FEATURE_CONTROL_LOCKED: u64 = 1 << 0;
const FEATURE_CONTROL_VMXON_ENABLED: u64 = 1 << 2;
const CR4_VMXE: u64 = 1 << 13;

pub unsafe fn check_vmx_support() -> Result<(), &'static str> {
    // TODO: CPUID check (ECX bit 5)
    
    let feat = rdmsr(MSR_IA32_FEATURE_CONTROL);
    
    if (feat & FEATURE_CONTROL_LOCKED) == 0 {
        return Err("IA32_FEATURE_CONTROL not locked");
    }
    if (feat & FEATURE_CONTROL_VMXON_ENABLED) == 0 {
        return Err("VMX disabled by BIOS");
    }
    
    // Activer CR4.VMXE
    let cr4 = read_cr4();
    write_cr4(cr4 | CR4_VMXE);
    
    Ok(())
}