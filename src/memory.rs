use crate::utils::shared::SharedPointer;

pub struct Memory {
    memory: SharedPointer<u8>,
}

impl Memory {
    pub fn new(memory: usize) -> Self {
        Memory {
            memory: SharedPointer::new("VcoreVriscMainMemory", memory).unwrap(),
        }
    }
}
