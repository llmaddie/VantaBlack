// src/vmx.rs
use crate::intel::*;
use crate::ffi::{alloc_zeroed_page, free_page, virt_to_phys};
use crate::utils::pr_alert;
use core::ptr;

/// VMXON region allouée et initialisée
pub struct VmxonRegion {
    va: *mut u8,
    pa: u64,
}

impl VmxonRegion {
    pub unsafe fn new() -> Result<Self, VmxError> {
        let va = alloc_zeroed_page();
        if va.is_null() {
            return Err(VmxError::AllocFailed);
        }
        
        let pa = virt_to_phys(va);
        if (pa & 0xFFF) != 0 {
            free_page(va);
            return Err(VmxError::Misaligned);
        }
        
        // Écriture revision ID (obligatoire avant VMXON)
        let vmx_basic = rdmsr(0x480);
        ptr::write_volatile(va as *mut u32, vmx_basic as u32);
        
        pr_alert!("VMXON region: PA={:#x}, rev={:#x}\n", pa, vmx_basic as u32);
        
        Ok(Self { va, pa })
    }
    
    pub unsafe fn activate(&self) -> Result<(), VmxError> {
        vmxon(self.pa).map_err(|e| match e {
            VmxInstructionError::VmFailInvalid => VmxError::VmxonInvalid,
            VmxInstructionError::VmFailValid => VmxError::VmxonError,
        })
    }
    
    pub unsafe fn deactivate(&self) {
        vmxoff();
    }
    
    pub fn physical_address(&self) -> u64 {
        self.pa
    }
}

impl Drop for VmxonRegion {
    fn drop(&mut self) {
        unsafe { free_page(self.va); }
    }
}

/// VMCS region
pub struct VmcsRegion {
    va: *mut u8,
    pa: u64,
}

impl VmcsRegion {
    pub unsafe fn new() -> Result<Self, VmxError> {
        let va = alloc_zeroed_page();
        if va.is_null() {
            return Err(VmxError::AllocFailed);
        }
        
        let pa = virt_to_phys(va);
        if (pa & 0xFFF) != 0 {
            free_page(va);
            return Err(VmxError::Misaligned);
        }
        
        let vmx_basic = rdmsr(0x480);
        ptr::write_volatile(va as *mut u32, vmx_basic as u32);
        
        Ok(Self { va, pa })
    }
    
    pub unsafe fn clear(&self) -> Result<(), VmxError> {
        vmclear(self.pa).map_err(|_| VmxError::VmclearFailed)
    }
    
    pub unsafe fn load(&self) -> Result<(), VmxError> {
        vmptrld(self.pa).map_err(|_| VmxError::VmptrldFailed)
    }
    
    pub unsafe fn write(&self, field: u64, value: u64) -> Result<(), VmxError> {
        vmwrite(field, value).map_err(|_| VmxError::VmwriteFailed)
    }
    
    pub unsafe fn read(&self, field: u64) -> Result<u64, VmxError> {
        vmread(field).map_err(|_| VmxError::VmreadFailed)
    }
    
    pub fn physical_address(&self) -> u64 {
        self.pa
    }
}

impl Drop for VmcsRegion {
    fn drop(&mut self) {
        unsafe { free_page(self.va); }
    }
}

#[derive(Debug)]
pub enum VmxError {
    AllocFailed,
    Misaligned,
    VmxonInvalid,
    VmxonError,
    VmclearFailed,
    VmptrldFailed,
    VmwriteFailed,
    VmreadFailed,
}