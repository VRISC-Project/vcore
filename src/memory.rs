use crate::{
    utils::shared::SharedPointer,
    vrisc::vcore::{BitOptions, FlagRegFlag},
};

pub enum AddressError {
    OverSized(u64),
    WrongPrivilege,
}

pub struct Memory {
    memory: SharedPointer<u8>,
}

impl Memory {
    pub fn new(memory: usize) -> Self {
        Memory {
            memory: SharedPointer::new("VcoreVriscMainMemory".to_string(), memory).unwrap(),
        }
    }

    pub fn bind(memory: usize) -> Self {
        Memory {
            memory: SharedPointer::bind("VcoreVriscMainMemory".to_string(), memory).unwrap(),
        }
    }

    pub fn borrow(&self) -> &SharedPointer<u8> {
        &self.memory
    }

    pub fn borrow_mut(&mut self) -> &mut SharedPointer<u8> {
        &mut self.memory
    }

    pub fn address(&mut self, addr: u64, flag: u64) -> Result<u64, AddressError> {
        let target = if flag.bit_get(FlagRegFlag::PagingEnabled) {
            todo!();
        } else {
            addr
        };
        if target >= self.memory.size() as u64 {
            Err(AddressError::OverSized(target))
        } else if flag.bit_get(FlagRegFlag::Privilege) && !((addr & (1 << 63)) != 0) {
            Err(AddressError::WrongPrivilege)
        } else {
            Ok(target)
        }
    }
}
