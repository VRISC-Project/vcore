use std::{collections::HashMap, sync::mpsc::Sender};

use crate::utils::shared::SharedPointer;

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
                if let Some(port) = self.intport.at_mut(0).core_get() {
                    sender.send(PortRequest::Interrupt(port as u16));
                }
                if !((self.reqport.at(0) >> 16) != 0) {
                    continue;
                }
                *self.reqport.at_mut(0) = (port_id as u32) + (*self.reqport.at(0) & 0xffff0000);
                while (self.reqport.at(0) >> 16) != 0 {}
                *self.reqport.at_mut(0) = 0 + (*self.reqport.at(0) & 0xffff0000);
                sender.send(PortRequest::Link(port_id)).unwrap();
                self.ports.insert(
                    port_id,
                    SharedPointer::new(format!("VcoreIOPort{}", port_id), 1).unwrap(),
                );
                port_id += 1;
            }
        }
    }

    pub fn do_solid_ports_services(
        ports: &mut Vec<Vec<SharedPointer<IOPortBuffer>>>,
        mut startflgs: Vec<SharedPointer<(bool, u64)>>,
    ) {
        for core in ports {
            // port 0: 设备连接端口
            // 不在这里实现
            // port 1: 多核唤醒
            if let Some(data) = core[1].at_mut(0).device_get() {
                let (core, ip) = {
                    let d1 = data & 0xffff_ffff;
                    let data = data >> 32;
                    (d1, data)
                };
                startflgs[core as usize].write(0, (true, ip));
            }
        }
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
