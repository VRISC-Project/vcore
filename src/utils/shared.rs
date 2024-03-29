use core::slice;
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
use std::mem::size_of;
use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};
#[cfg(target_os = "linux")]
use std::{mem::size_of, num::NonZeroUsize};

#[cfg(target_os = "windows")]
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(target_os = "windows")]
use winapi::{
    ctypes::c_void,
    um::memoryapi::{
        CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    },
    um::{errhandlingapi::GetLastError, minwinbase::SECURITY_ATTRIBUTES, winnt::PAGE_READWRITE},
};

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub enum MapError {
    CreateFileMappingError(u32),
    MapViewOfFileError(u32),
    OpenFileMappingError(u32),
}

#[derive(Debug)]
pub enum AssignError {
    IndexOutOfSize,
}

#[derive(Debug)]
/// ## 共享内存的指针
///
/// 将linux、winows、mac三个平台上的共享内存功能封装。
///
/// > 并不是智能指针
pub struct SharedPointer<T> {
    pub pointer: *mut T,
    size: usize,
    name: String,
    #[cfg(target_os = "linux")]
    fd: i32,
    #[cfg(target_os = "windows")]
    hdl: *mut c_void,
}

unsafe impl<T> Send for SharedPointer<T> {}
unsafe impl<T> Sync for SharedPointer<T> {}

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
        let mut namev = Vec::new();
        let namew = {
            for x in name.as_bytes() {
                namev.push(*x as u16);
            }
            namev.as_slice()
        };
        let hdl = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                0 as *mut SECURITY_ATTRIBUTES,
                PAGE_READWRITE,
                ((size * size_of::<T>()) >> 32) as u32,
                (size * size_of::<T>()) as u32,
                namew.as_ptr(),
            )
        };
        if hdl == (0 as *mut c_void) {
            return Err(MapError::CreateFileMappingError(unsafe { GetLastError() }));
        }
        let addr = unsafe { MapViewOfFile(hdl, FILE_MAP_ALL_ACCESS, 0, 0, 0) };
        if addr == (0 as *mut c_void) {
            return Err(MapError::MapViewOfFileError(unsafe { GetLastError() }));
        }
        Ok(SharedPointer {
            pointer: addr as *mut T,
            size,
            name,
            hdl,
        })
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
        let mut namev = Vec::new();
        for x in name.as_bytes() {
            namev.push(*x as u16);
        }
        let namew = namev.as_slice();
        let hdl = unsafe { OpenFileMappingW(PAGE_READWRITE, false as i32, namew.as_ptr()) };
        if hdl == (0 as *mut c_void) {
            return Err(MapError::CreateFileMappingError(unsafe { GetLastError() }));
        }
        let addr = unsafe { MapViewOfFile(hdl, FILE_MAP_ALL_ACCESS, 0, 0, 0) };
        if addr == (0 as *mut c_void) {
            return Err(MapError::MapViewOfFileError(unsafe { GetLastError() }));
        }
        Ok(SharedPointer {
            pointer: addr as *mut T,
            size,
            name,
            hdl,
        })
    }

    #[cfg(target_os = "macos")]
    pub fn bind(name: String, size: usize) -> Result<Self, Errno> {
        todo!();
    }
}

impl<T> Drop for SharedPointer<T> {
    #[cfg(target_os = "linux")]
    fn drop(&mut self) {
        unsafe { sys::mman::munmap(self.pointer as *mut c_void, self.size).unwrap() };
        unistd::close(self.fd).unwrap();
        self.pointer = 0 as *mut T;
    }

    #[cfg(target_os = "windows")]
    fn drop(&mut self) {
        use winapi::um::handleapi::CloseHandle;

        unsafe {
            UnmapViewOfFile(self.pointer as *mut c_void);
            CloseHandle(self.hdl);
        }
        self.pointer = 0 as *mut T;
    }
}

impl<T> Clone for SharedPointer<T> {
    fn clone(&self) -> Self {
        SharedPointer::<T>::bind(self.name.clone(), self.size()).unwrap()
    }
}

pub trait Addressable<T>: DerefMut {
    fn slice<'a>(&self, addr: u64, len: u64) -> &'a [T];
    fn slice_mut<'a>(&mut self, addr: u64, len: u64) -> &'a mut [T];

    fn at<'a>(&self, addr: u64) -> &'a T;
    fn at_mut<'a>(&mut self, addr: u64) -> &'a mut T;

    fn write(&mut self, addr: u64, t: T);
    fn write_slice(&mut self, addr: u64, s: &[T]);
}

impl<T> Addressable<T> for SharedPointer<T> {
    #[inline]
    fn slice<'a>(&self, addr: u64, mut len: u64) -> &'a [T] {
        if ((addr + len) as usize) > self.size {
            len = (self.size as u64) - addr;
        }
        unsafe { slice::from_raw_parts(self.pointer.add(addr as usize), len as usize) }
    }

    #[inline]
    fn slice_mut<'a>(&mut self, addr: u64, mut len: u64) -> &'a mut [T] {
        if ((addr + len) as usize) > self.size {
            len = (self.size as u64) - addr;
        }
        unsafe { slice::from_raw_parts_mut(((self.pointer as u64) + addr) as *mut T, len as usize) }
    }

    #[inline]
    fn at<'a>(&self, addr: u64) -> &'a T {
        unsafe { &*self.pointer.add(addr as usize) }
    }

    #[inline]
    fn at_mut<'a>(&mut self, addr: u64) -> &'a mut T {
        unsafe { &mut *self.pointer.add(addr as usize) }
    }

    #[inline]
    fn write(&mut self, addr: u64, t: T) {
        if (addr as usize) < self.size {
            unsafe {
                *self.pointer.add(addr as usize) = t;
            }
        }
    }

    #[inline]
    fn write_slice(&mut self, addr: u64, s: &[T]) {
        if (addr as usize) + s.len() < self.size {
            unsafe {
                self.pointer
                    .add(addr as usize)
                    .copy_from(s.as_ptr(), s.len())
            }
        }
    }
}

impl<T> Deref for SharedPointer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.at(0)
    }
}

impl<T> DerefMut for SharedPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.at_mut(0)
    }
}

impl<T> SharedPointer<T> {
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}
