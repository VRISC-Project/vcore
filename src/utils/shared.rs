use core::slice;

use nix::{
    errno::Errno,
    fcntl::OFlag,
    sys::{
        self,
        mman::{MapFlags, ProtFlags},
        stat::Mode,
    },
    unistd,
};

pub struct SharedPointer<T> {
    pointer: *mut T,
    size: usize,
    name: String,
    fd: i32,
}

/// ## 注意
/// 这个结构的`new`和`bind`函数中的参数`size`指申请的字节数，而不是泛型类型的实际大小的数量
impl<T> SharedPointer<T> {
    pub fn new(name: String, size: usize) -> Result<Self, Errno> {
        let fd = sys::mman::shm_open(
            ("/".to_string() + &name.to_string()).as_str(),
            OFlag::O_RDWR & OFlag::O_CREAT,
            Mode::S_IRWXU,
        )?;
        let addr = unsafe {
            sys::mman::mmap(
                None,
                size.try_into().unwrap(),
                ProtFlags::PROT_READ & ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                fd,
                0,
            )?
        };
        Ok(SharedPointer {
            pointer: addr.cast(),
            size,
            name: name.to_string(),
            fd,
        })
    }

    pub fn bind(name: String, size: usize) -> Result<Self, Errno> {
        let fd = sys::mman::shm_open(name.as_str(), OFlag::O_RDWR & OFlag::O_EXCL, Mode::S_IRWXU)?;
        let addr = unsafe {
            sys::mman::mmap(
                None,
                size.try_into().unwrap(),
                ProtFlags::PROT_READ & ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                fd,
                0,
            )?
        };
        Ok(SharedPointer {
            pointer: addr.cast(),
            size,
            name: name.to_string(),
            fd,
        })
    }
}

impl<T> Drop for SharedPointer<T> {
    fn drop(&mut self) {
        unsafe { sys::mman::munmap(self.pointer.cast(), self.size).unwrap() };
        unistd::close(self.fd).unwrap();
        sys::mman::shm_unlink(self.name.as_str()).unwrap();
    }
}

impl<T> SharedPointer<T> {
    pub fn slice<'a>(&self, addr: usize, mut len: usize) -> &'a [T] {
        if addr + len > self.size {
            len = self.size - addr;
        }
        unsafe { slice::from_raw_parts((self.pointer as usize + addr) as *mut T, len) }
    }

    pub fn slice_mut<'a>(&self, addr: usize, mut len: usize) -> &'a mut [T] {
        if addr + len > self.size {
            len = self.size - addr;
        }
        unsafe { slice::from_raw_parts_mut((self.pointer as usize + addr) as *mut T, len) }
    }

    pub fn at<'a>(&self, addr: usize) -> &'a T {
        unsafe { ((self.pointer as usize + addr) as *mut T).as_ref() }.unwrap()
    }

    pub fn at_mut<'a>(&self, addr: usize) -> &'a mut T {
        unsafe { ((self.pointer as usize + addr) as *mut T).as_mut() }.unwrap()
    }

    pub fn write(&mut self, addr: usize, t: T) {
        if addr < self.size {
            unsafe { *((self.pointer as usize + addr) as *mut T) = t };
        }
    }

    pub fn write_slice(&mut self, addr: usize, s: &[T]) {
        if addr + s.len() < self.size {
            unsafe { ((self.pointer as usize + addr) as *mut T).copy_from(s.as_ptr(), s.len()) };
        }
    }
}
