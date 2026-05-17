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

impl kernel::Module for MyModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {

        let cpu = cpuid::cpuid(0, 0);

        pr_info!("EAX: 0x{:08x}\n", cpu.eax);
        pr_info!("EBX: 0x{:08x}\n", cpu.ebx);
        pr_info!("ECX: 0x{:08x}\n", cpu.ecx);
        pr_info!("EDX: 0x{:08x}\n", cpu.edx);

        let vmx_bit = (cpu.ecx >> 5) & 1;
        if vmx_bit == 1 {
            pr_info!("vantablack: VMX intel-VT-X is supported\n")
        } else {
            pr_info!("vantablack: VMX Intel Vt-X not supported\n")
        }
    Ok(MyModule)
    }
}