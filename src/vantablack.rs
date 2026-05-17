mod cpuid;

use kernel::prelude::*;

module! {
    type: MyModule,
    name: "my_cpuid",
    author: "toi",
    description: "CPUID test",
    license: "GPL",
}

struct MyModule;

impl kernel::Module for MyModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {

        let cpu = cpuid::cpuid(0, 0);

        pr_info!("EAX: {}\n", cpu.eax);
        pr_info!("EBX: {}\n", cpu.ebx);
        pr_info!("ECX: {}\n", cpu.ecx);
        pr_info!("EDX: {}\n", cpu.edx);

        Ok(MyModule)
    }
}