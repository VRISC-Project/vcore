use std::collections::HashMap;

use crate::utils::shared::SharedPointer;

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

    pub fn device_bind(id: u64) -> Option<SharedPointer<DMAObject>> {
        Some(SharedPointer::<DMAObject>::bind(format!("VcoreDMA{}Obj", id), 1).unwrap())
    }

    pub fn create_new(&mut self) -> u64 {
        let res = self.dma_count;
        let dmaobj = SharedPointer::<DMAObject>::new(format!("VcoreDMA{}Obj", res), 1).unwrap();
        self.dmas.insert(res, dmaobj);
        self.dma_count += 1;
        res
    }

    pub fn set_start(&mut self, id: u64, start: u64) {
        self.dmas.get_mut(&id).unwrap().at_mut(0).start = start;
    }

    pub fn set_length(&mut self, id: u64, length: u64) {
        self.dmas.get_mut(&id).unwrap().at_mut(0).length = length;
    }

    pub fn set_read(&mut self, id: u64, read: u64) {
        self.dmas.get_mut(&id).unwrap().at_mut(0).read = read != 0;
    }

    pub fn set_write(&mut self, id: u64, write: u64) {
        self.dmas.get_mut(&id).unwrap().at_mut(0).write = write != 0;
    }

    pub fn remove(&mut self, id: u64) {
        self.dmas.remove(&id);
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
