use std::{
    alloc,
    alloc::Layout,
    fs,
    io::Read,
    sync::{Arc, Mutex},
};

///# 内存模块
pub struct Memory {
    pub memory: *mut u8,
    size: usize,

    count: Arc<Mutex<usize>>,
}

impl Memory {
    ///初始化一个大小为`size`的内存
    pub fn new(size: usize) -> Self {
        let memory;
        unsafe {
            memory = alloc::alloc(Layout::from_size_align(size, 8).unwrap());
        }
        if memory.is_null() {
            panic!("Failed to allocate {} byte memory.", size);
        }
        Memory {
            memory,
            size,
            count: Arc::new(Mutex::new(1usize)),
        }
    }

    ///加载固件文件中的程序代码
    pub fn load_firmware(&mut self, path: &str) {
        let mut firmware;
        if let Ok(file) = fs::File::open(&path) {
            firmware = file;
        } else {
            panic!("Failed to load firmware file {}.", path);
        }
        let mut buffer = [0u8; 32];
        let mut offset = 0u64;
        while let Ok(len) = firmware.read(&mut buffer) {
            if len == 0 {
                break;
            }
            unsafe {
                for i in 0..len {
                    *(((self.memory as u64) + offset + i as u64) as *mut u8) = buffer[i as usize];
                }
            }
            offset += len as u64;
            println!("loaded section {}.", offset);
        }
        println!("Found firmware \"{}\".", path);
    }

    pub fn address(&mut self, addr: u64) -> u64 {
        todo!();
    }

    pub fn read_byte(&self, addr: u64) -> u8 {
        unsafe { *((self.memory as u64 + addr) as *mut u8) }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        {
            *self.count.lock().unwrap() -= 1;
        }
        if *self.count.lock().unwrap() == 0 {
            unsafe {
                alloc::dealloc(self.memory, Layout::from_size_align(self.size, 8).unwrap());
            }
        }
    }
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        {
            *self.count.lock().unwrap() += 1;
        }
        Self {
            memory: self.memory,
            size: self.size,
            count: Arc::clone(&self.count),
        }
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}
