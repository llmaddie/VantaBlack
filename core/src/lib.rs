//! CPUID instruction to the Kernel Module 
#![allow(missing_docs)]
mod cpuid;

use kernel::prelude::*;

module! {
    type: MyModule,
    name: "my_cpuid",
    authors: ["toi"],
    description: "CPUID test",
    license: "GPL",
}

struct MyModule;

#[inline]
unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;

    unsafe { // wrap the inline assembly to his own unsafe 
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
    }

    ((high as u64) << 32) | (low as u64)
}
impl kernel::Module for MyModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {

        let cpu = cpuid::cpuid(0, 0);

        pr_info!("EAX: 0x{:08x}\n", cpu.eax);
        pr_info!("EBX: 0x{:08x}\n", cpu.ebx);
        pr_info!("ECX: 0x{:08x}\n", cpu.ecx);
        pr_info!("EDX: 0x{:08x}\n", cpu.edx);

        let msr_value = unsafe { read_msr(0x3A) };
            if (msr_value & 1) == 0 {
                pr_alert!("vantablack: MSR lock bit is 0, VMX is disabled in BIOS\n");
                return Err(kernel::error::code::ENODEV);
            }
            if (msr_value & (1 << 2)) == 0 {
                pr_alert!("vantablack: VMX outside SMX is disabled\n");
                return Err(kernel::error::code::ENODEV);
            }
            pr_alert!("vantablack: MSR check passed. BIOS allow VMX operation\n");

        let vmx_bit = (cpu.ecx >> 5) & 1;
        if vmx_bit == 1 {
            pr_info!("vantablack: VMX intel-VT-X is supported\n")
        } else {
            pr_info!("vantablack: VMX Intel Vt-X not supported\n")
        }
    Ok(MyModule)
    }
}