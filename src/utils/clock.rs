#[cfg(target_os = "linux")]
use nix::libc::{gettimeofday, timeval, timezone};
#[cfg(target_os = "windows")]
use winapi::um::timeapi::timeGetTime;

/// ## vcore内部时钟
pub struct Clock {
    cycle: u8, // ms

    last_usec: i64,
}

impl Clock {
    /// ### 创建一个周期为`cycle`的时钟
    pub fn new(cycle: u8) -> Self {
        Clock {
            cycle,
            last_usec: 0,
        }
    }

    #[cfg(target_os = "linux")]
    /// ## 判断是否已经过了一个周期
    ///
    /// 返回true时要产生一个中断
    pub fn hit(&mut self) -> bool {
        let mut tv = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        unsafe { gettimeofday(&mut tv, 0 as *mut timezone) };
        if tv.tv_sec * 1000_000 + tv.tv_usec - self.last_usec >= self.cycle as i64 * 1000 {
            self.last_usec = tv.tv_sec * 1000_000 + tv.tv_usec;
            true
        } else {
            false
        }
    }

    #[cfg(target_os = "windows")]
    pub fn hit(&mut self) -> bool {
        let t = unsafe { timeGetTime() };
        if t * 1000 - self.last_usec as u32 >= self.cycle as u32 * 1000 {
            self.last_usec = t as i64 * 1000;
            true
        } else {
            false
        }
    }

    #[cfg(target_os = "macos")]
    pub fn hit(&mut self) -> bool {
        todo!();
    }
}
