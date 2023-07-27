use core::slice;
use std::{mem::size_of, num::NonZeroUsize};

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub enum MapError {}

#[derive(Debug)]
pub enum AssignError {
    IndexOutOfSize,
}

pub struct SharedPointer<T> {
    pub pointer: *mut T,
    size: usize,
    name: String,
    fd: i32,
}

/// ## 注意
/// 这个结构的`new`和`bind`函数中的参数`size`指申请的字节数，而不是泛型类型的实际大小的数量
impl<T> SharedPointer<T> {
    #[cfg(target_os = "linux")]
    pub fn new(name: String, size: usize) -> Result<Self, Errno> {
        if size == 0 {
            panic!("The memory you are allocating sizes 0.");
        }
        let fd = sys::mman::shm_open(
            ("/".to_string() + &name).as_str(),
            OFlag::O_RDWR | OFlag::O_CREAT,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )?;
        unistd::ftruncate(fd, (size * size_of::<T>()) as i64)?;
        let addr = unsafe {
            sys::mman::mmap(
                None,
                NonZeroUsize::new_unchecked(size * size_of::<T>()),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                fd,
                0,
            )?
        };
        Ok(SharedPointer {
            pointer: addr as *mut T,
            size,
            name,
            fd,
        })
    }

    #[cfg(target_os = "windows")]
    pub fn new(name: String, size: usize) -> Result<Self, MapError> {
        todo!();
    }

    #[cfg(target_os = "macos")]
    pub fn new(name: String, size: usize) -> Result<Self, Errno> {
        todo!();
    }

    #[cfg(target_os = "linux")]
    pub fn bind(name: String, size: usize) -> Result<Self, Errno> {
        if size == 0 {
            panic!("The memory you are allocating sizes 0.");
        }
        let fd = sys::mman::shm_open(
            ("/".to_string() + &name).as_str(),
            OFlag::O_RDWR | OFlag::O_EXCL,
            Mode::from_bits(0).unwrap(),
        )?;
        let addr = unsafe {
            sys::mman::mmap(
                None,
                NonZeroUsize::new_unchecked(size * size_of::<T>()),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                fd,
                0,
            )?
        };
        Ok(SharedPointer {
            pointer: addr as *mut T,
            size,
            name,
            fd,
        })
    }

    #[cfg(target_os = "windows")]
    pub fn bind(name: String, size: usize) -> Result<Self, MapError> {
        todo!();
    }

    #[cfg(target_os = "macos")]
    pub fn bind(name: String, size: usize) -> Result<Self, Errno> {
        todo!();
    }

    pub fn assign(&mut self, index: usize, t: T) -> Result<(), AssignError> {
        if index * size_of::<T>() > self.size {
            return Err(AssignError::IndexOutOfSize);
        }
        let p = &t as *const T as *const u8;
        unsafe {
            for i in 0..size_of::<T>() {
                *(self.pointer.add(index * size_of::<T>() + i) as *mut u8) = *p.add(i);
            }
        };
        Ok(())
    }
}

impl<T> Drop for SharedPointer<T> {
    #[cfg(target_os = "linux")]
    fn drop(&mut self) {
        unsafe { sys::mman::munmap(self.pointer.cast(), self.size).unwrap() };
        unistd::close(self.fd).unwrap();
        // 写了这句会出现'ENOENT'错误
        // 不写这句会有小概率会在下次运行申请共享内存时发生'ENOENT'，
        // 没搞懂是怎么回事，不过先注释上目前没啥大毛病
        // sys::mman::shm_unlink(("/".to_string() + &self.name).as_str()).unwrap();
    }

    #[cfg(target_os = "windows")]
    fn drop(&mut self) {
        todo!();
    }
}

impl<T> SharedPointer<T> {
    pub fn slice<'a>(&self, addr: u64, mut len: u64) -> &'a [T] {
        if (addr + len) as usize > self.size {
            len = self.size as u64 - addr;
        }
        unsafe { slice::from_raw_parts(self.pointer.add(addr as usize), len as usize) }
    }

    pub fn slice_mut<'a>(&self, addr: u64, mut len: u64) -> &'a mut [T] {
        if (addr + len) as usize > self.size {
            len = self.size as u64 - addr;
        }
        unsafe { slice::from_raw_parts_mut((self.pointer as u64 + addr) as *mut T, len as usize) }
    }

    pub fn at<'a>(&self, addr: u64) -> &'a T {
        unsafe { &*self.pointer.add(addr as usize) }
    }

    pub fn at_mut<'a>(&self, addr: u64) -> &'a mut T {
        unsafe { &mut *self.pointer.add(addr as usize) }
    }

    pub fn write(&mut self, addr: u64, t: T) {
        if (addr as usize) < self.size {
            unsafe { *self.pointer.add(addr as usize) = t };
        }
    }

    pub fn write_slice(&mut self, addr: u64, s: &[T]) {
        if addr as usize + s.len() < self.size {
            unsafe {
                self.pointer
                    .add(addr as usize)
                    .copy_from(s.as_ptr(), s.len())
            };
        }
    }
}

impl<T> SharedPointer<T> {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
