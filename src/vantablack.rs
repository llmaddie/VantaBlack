#![doc = "vantablack kernel module"]

use kernel::prelude::*;

module! {
    type: RustMinimal,
    name: "vantablack",
    authors: ["maddie"],
    description: "module kernel minimal on rust",
    license: "GPL",
}

struct RustMinimal;

impl kernel::Module for RustMinimal {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal module load\n");
        Ok(RustMinimal)
    }
}

impl Drop for RustMinimal {
    fn drop(&mut self) {
        pr_info!("Rust module unload\n");
    }
}