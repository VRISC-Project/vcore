use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crate::utils::shared::{Addressable, SharedPointer};

use super::dma::{DMADevice, DMAStatus, DirectMemoryAccess};

/// ## 核心IO控制器
pub struct IOController {
    /// ## 请求端口
    ///
    /// (port: u16, lock: bool)
    /// port初始值为0， lock初始值为false；
    /// 请求时将锁锁住，等待thr_pushreq为请求分配端口，写入port中；
    /// 然后将锁解锁，连接到指定port。
    ///
    /// 由于无法为(u16, bool)实现Send trait，使用u32代替
    pub reqport: SharedPointer<u32>,

    /// ## 中断端口
    ///
    /// 由于不需要保证百分百响应，并且使用频率较高，使用IOPortBuffer即可。
    pub intport: SharedPointer<IOPortBuffer>,

    /// ## 端口
    ///
    /// 端口号对应的端口结构存在这里。
    pub ports: HashMap<u16, SharedPointer<IOPortBuffer>>,

    /// ## 端口请求分配器
    ///
    /// 通过这个Sender把分配的端口发送给某个核心，
    /// vec中每个Sender对应一个核心的Reciever。
    pub port_deliver: Vec<Sender<PortRequest>>,
}

pub enum PortRequest {
    Link(u16),
    Interrupt(u16),
}

unsafe impl Send for IOController {}
unsafe impl Sync for IOController {}

impl IOController {
    pub fn new(delivers: Vec<Sender<PortRequest>>) -> Self {
        Self {
            reqport: SharedPointer::<u32>::new(String::from("VcoreIORequestPort"), 1).unwrap(),
            intport: SharedPointer::<IOPortBuffer>::new(String::from("VcoreInterruptPort"), 1)
                .unwrap(),
            ports: HashMap::new(),
            port_deliver: delivers,
        }
    }

    pub fn thr_dispatch_ioreq(&mut self) {
        self.reqport.write(0, 0);
        let mut port_id = 256u16;
        loop {
            for sender in self.port_deliver.iter_mut() {
                if let Some(port) = self.intport.core_get() {
                    sender.send(PortRequest::Interrupt(port as u16)).unwrap();
                }
                if (*self.reqport >> 16) == 0 {
                    continue;
                }
                *self.reqport = (port_id as u32) + (*self.reqport & 0xffff0000);
                while (*self.reqport >> 16) != 0 {
                    assert!(true); // 使while循环反复求值而不是死循环
                }
                *self.reqport = 0 + (*self.reqport & 0xffff0000);
                sender.send(PortRequest::Link(port_id)).unwrap();
                self.ports.insert(
                    port_id,
                    SharedPointer::new(format!("VcoreIOPort{}", port_id), 1).unwrap(),
                );
                if port_id == u16::MAX {
                    port_id = 256;
                } else {
                    port_id += 1;
                }
            }
        }
    }

    pub fn do_solid_ports_services(
        ports: &mut Vec<Vec<SharedPointer<IOPortBuffer>>>,
        mut startflgs: Vec<SharedPointer<(bool, u64)>>,
        dma_controller: Arc<RwLock<DirectMemoryAccess>>,
    ) {
        let mut dma_current = 0;
        let mut dma_opstatus = DMAStatus::None;
        for core in ports {
            // port 0: 设备连接端口
            // 不在这里实现
            // port 1: 多核唤醒
            if let Some(data) = core[1].device_get() {
                let (core, ip) = {
                    let d1 = data & 0xffff_ffff;
                    let data = data >> 32;
                    (d1, data)
                };
                startflgs[core as usize].write(0, (true, ip));
            }
            // port 2: dma管理
            if let Some(data) = core[2].device_get() {
                match data {
                    0 => {
                        dma_current = dma_controller.write().unwrap().create_new();
                        core[2].device_push(dma_current);
                    }
                    1 => {
                        dma_opstatus = DMAStatus::SetCurrentDMAId;
                    }
                    2 => {
                        dma_opstatus = DMAStatus::SetDMAStart;
                    }
                    3 => {
                        dma_opstatus = DMAStatus::SetDMALength;
                    }
                    4 => {
                        dma_opstatus = DMAStatus::SetDMARead;
                    }
                    5 => {
                        dma_opstatus = DMAStatus::SetDMAWrite;
                    }
                    6 => {
                        dma_controller.write().unwrap().remove(dma_current);
                    }
                    data => match dma_opstatus {
                        DMAStatus::SetCurrentDMAId => {
                            dma_current = data;
                        }
                        DMAStatus::SetDMAStart => {
                            dma_controller.write().unwrap().set_start(dma_current, data);
                        }
                        DMAStatus::SetDMALength => dma_controller
                            .write()
                            .unwrap()
                            .set_length(dma_current, data),
                        DMAStatus::SetDMARead => {
                            dma_controller.write().unwrap().set_read(dma_current, data);
                        }
                        DMAStatus::SetDMAWrite => {
                            dma_controller.write().unwrap().set_write(dma_current, data);
                        }
                        _ => (),
                    },
                }
            }
        }
    }
}

pub struct IODevice {
    port_id: u16,
    io_port: SharedPointer<IOPortBuffer>,
    int_port: SharedPointer<IOPortBuffer>,
}

impl IODevice {
    pub fn new() -> Self {
        let mut reqport =
            SharedPointer::<u32>::bind(String::from("VcoreIORequestPort"), 1).unwrap();
        reqport.write(0, 1 << 16);
        while *reqport & 0xffff == 0 {
            assert!(true);
        }
        let port_id = (*reqport & 0xffff) as u16;
        reqport.write(0, 0);
        let io_port =
            SharedPointer::<IOPortBuffer>::bind(format!("VcoreIOPort{}", port_id), 1).unwrap();
        Self {
            port_id,
            io_port,
            int_port: SharedPointer::<IOPortBuffer>::bind(String::from("VcoreInterruptPort"), 1)
                .unwrap(),
        }
    }
}

pub struct CharacterDevice {
    dev: IODevice,
}

impl CharacterDevice {
    pub fn new() -> Self {
        Self {
            dev: IODevice::new(),
        }
    }

    pub fn input(&mut self, content: &[u8]) {
        let mut count = 0;
        let mut data = 0u64;
        for byte in content {
            data |= (*byte as u64) << (count * 8);
            count += 1;
            if count == 8 {
                count = 0;
                self.dev.io_port.device_push(data);
            }
        }
        if count != 0 {
            self.dev.io_port.device_push(data);
        }
        self.dev.int_port.device_push(self.dev.port_id as u64);
    }

    pub fn output(&mut self) -> Option<Vec<u8>> {
        let mut v = Vec::new();
        while let Some(qw) = self.dev.io_port.device_get() {
            for i in 0..8 {
                v.push((qw >> (i * 8)) as u8);
            }
        }
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}

pub struct BlockDevice {
    dev: CharacterDevice,
    dma: Option<DMADevice>,
}

impl BlockDevice {
    pub fn new() -> Self {
        Self {
            dev: CharacterDevice::new(),
            dma: None,
        }
    }

    pub fn get_dev_mut(&mut self) -> &mut CharacterDevice {
        &mut self.dev
    }

    pub fn get_dma(&self) -> &Option<DMADevice> {
        &self.dma
    }

    pub fn get_dma_mut(&mut self) -> &mut Option<DMADevice> {
        &mut self.dma
    }
}

pub struct IOPortBuffer {
    ifront: usize,
    irear: usize,
    ibuffer: [u64; 4096],

    ofront: usize,
    orear: usize,
    obuffer: [u64; 4096],
}

impl IOPortBuffer {
    pub fn core_push(&mut self, data: u64) {
        self.obuffer[self.orear] = data;
        self.orear += 1;
        if self.orear == 4096 {
            self.orear = 0;
        }
    }

    pub fn device_push(&mut self, data: u64) {
        self.ibuffer[self.irear] = data;
        self.irear += 1;
        if self.irear == 4096 {
            self.irear = 0;
        }
    }

    pub fn core_get(&mut self) -> Option<u64> {
        if self.ifront == self.irear {
            return None;
        }
        let x = self.ibuffer[self.ifront];
        self.ifront += 1;
        if self.ifront == 4096 {
            self.ifront = 0;
        }
        Some(x)
    }

    pub fn device_get(&mut self) -> Option<u64> {
        if self.ofront == self.orear {
            return None;
        }
        let x = self.obuffer[self.ofront];
        self.ofront += 1;
        if self.ofront == 4096 {
            self.ofront = 0;
        }
        Some(x)
    }
}
