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
    pub fn new(name: &str, size: usize) -> Result<Self, Errno> {
        let fd = sys::mman::shm_open(("/".to_string() + &name.to_string()).as_str(), OFlag::O_RDWR & OFlag::O_CREAT, Mode::S_IRWXU)?;
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

    pub fn bind(name: &str, size: usize) -> Result<Self, Errno> {
        let fd = sys::mman::shm_open(name, OFlag::O_RDWR & OFlag::O_EXCL, Mode::S_IRWXU)?;
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
