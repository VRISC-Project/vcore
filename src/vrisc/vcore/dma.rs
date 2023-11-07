use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::utils::{
    memory::Memory,
    shared::{Addressable, SharedPointer},
};

pub struct DirectMemoryAccess {
    dmas: HashMap<u64, SharedPointer<DMAObject>>,
    dma_count: u64,
}

impl DirectMemoryAccess {
    pub fn new() -> Self {
        Self {
            dmas: HashMap::new(),
            dma_count: 1,
        }
    }

    pub fn device_bind(id: u64) -> Option<(SharedPointer<DMAObject>, SharedPointer<u8>)> {
        let dmaobj = SharedPointer::<DMAObject>::bind(format!("VcoreDMA{}Obj", id), 1).unwrap();
        let size = dmaobj.start + dmaobj.length;
        Some((
            dmaobj,
            SharedPointer::<u8>::bind(String::from("VcoreVriscMainMemory"), size as usize).unwrap(),
        ))
    }

    pub fn create_new(&mut self) -> u64 {
        let res = self.dma_count;
        let dmaobj = SharedPointer::<DMAObject>::new(format!("VcoreDMA{}Obj", res), 1).unwrap();
        self.dmas.insert(res, dmaobj);
        self.dma_count += 1;
        res
    }

    pub fn set_start(&mut self, id: u64, start: u64) {
        self.dmas.get_mut(&id).unwrap().start = start;
    }

    pub fn set_length(&mut self, id: u64, length: u64) {
        self.dmas.get_mut(&id).unwrap().length = length;
    }

    pub fn set_read(&mut self, id: u64, read: u64) {
        self.dmas.get_mut(&id).unwrap().read = read != 0;
    }

    pub fn set_write(&mut self, id: u64, write: u64) {
        self.dmas.get_mut(&id).unwrap().write = write != 0;
    }

    pub fn remove(&mut self, id: u64) {
        self.dmas.remove(&id);
    }
}

pub struct DMADevice {
    obj: SharedPointer<DMAObject>,
    mem: Memory,
}

impl DMADevice {
    pub fn new(dma_id: u64) -> Self {
        let dmaobj = SharedPointer::<DMAObject>::bind(format!("VcoreDMA{}Obj", dma_id), 1).unwrap();
        let sz = dmaobj.start + dmaobj.length;
        Self {
            obj: dmaobj,
            mem: Memory::bind(sz as usize),
        }
    }
}

impl Addressable<u8> for DMADevice {
    fn slice<'a>(&self, addr: u64, len: u64) -> &'a [u8] {
        if self.obj.start + addr >= self.obj.length || len >= self.obj.length {
            panic!("In DMA accessing: slicing section out of size.");
        }
        self.mem.borrow().slice(self.obj.start + addr, len)
    }

    fn slice_mut<'a>(&mut self, addr: u64, len: u64) -> &'a mut [u8] {
        if self.obj.start + addr >= self.obj.length || len >= self.obj.length {
            panic!("In DMA accessing: slicing section out of size.");
        }
        self.mem.borrow_mut().slice_mut(self.obj.start + addr, len)
    }

    fn at<'a>(&self, addr: u64) -> &'a u8 {
        if self.obj.start + addr >= self.obj.length {
            panic!("In DMA accessing: slicing section out of size.");
        }
        self.mem.borrow().at(self.obj.start + addr)
    }

    fn at_mut<'a>(&mut self, addr: u64) -> &'a mut u8 {
        if self.obj.start + addr >= self.obj.length {
            panic!("In DMA accessing: slicing section out of size.");
        }
        self.mem.borrow_mut().at_mut(self.obj.start + addr)
    }

    fn write(&mut self, addr: u64, t: u8) {
        if self.obj.start + addr >= self.obj.length {
            panic!("In DMA accessing: slicing section out of size.");
        }
        self.mem.borrow_mut().write(self.obj.start + addr, t);
    }

    fn write_slice(&mut self, addr: u64, s: &[u8]) {
        if self.obj.start + addr >= self.obj.length {
            panic!("In DMA accessing: slicing section out of size.");
        }
        self.mem.borrow_mut().write_slice(self.obj.start + addr, s);
    }
}

impl Deref for DMADevice {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        self.at(0)
    }
}

impl DerefMut for DMADevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.at_mut(0)
    }
}

/// ## dma状态
///
/// 用于io控制器的固定端口服务
/// 确定当前设置的状态并调用对应函数
pub enum DMAStatus {
    None,
    SetCurrentDMAId,
    SetDMAStart,
    SetDMALength,
    SetDMARead,
    SetDMAWrite,
    RemoveDMA,
}

pub struct DMAObject {
    /// 起始物理地址
    start: u64,
    length: u64,
    read: bool,
    write: bool,
}
