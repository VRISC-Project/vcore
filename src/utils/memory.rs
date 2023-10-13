use std::collections::{BTreeMap, BinaryHeap};

use crate::{
    utils::shared::SharedPointer,
    vrisc::vcore::{BitOptions, FlagRegFlag},
};

#[derive(Debug)]
pub enum AddressError {
    OverSized(u64),
    WrongPrivilege,
    Ineffective,
    Unreadable,
    Unwritable,
}

#[derive(Debug, Clone)]
/// ## vcore的内存
pub struct Memory {
    memory: SharedPointer<u8>,

    /// ## 寻址缓冲
    ///
    /// 由于寻址操作非常耗时，所以不希望同一地址被重复寻址，使用BTreeMap存储
    /// 逻辑地址-物理地址对，逻辑地址已存在无需再重复寻址
    address_buffer: BTreeMap<u64, u64>,

    /// ## 计数筛选器
    ///
    /// 由于访问Map同样有点耗时，需要使address_buffer的大小维持在固定的大小，
    /// 使用一个小根堆快速选取使用次数最少的的逻辑地址，在超过固定大小时从address_buffer中移除。
    ///
    /// > * 新手保护：保证刚刚加入的地址不会被立即移除
    filter: BinaryHeap<UsageCounter>,
}

impl Memory {
    pub fn new(memory: usize) -> Self {
        Memory {
            memory: SharedPointer::new("VcoreVriscMainMemory".to_string(), memory).unwrap(),
            address_buffer: BTreeMap::new(),
            filter: BinaryHeap::new(),
        }
    }

    pub fn bind(memory: usize) -> Self {
        Memory {
            memory: SharedPointer::bind("VcoreVriscMainMemory".to_string(), memory).unwrap(),
            address_buffer: BTreeMap::new(),
            filter: BinaryHeap::new(),
        }
    }

    #[inline]
    pub fn borrow(&self) -> &SharedPointer<u8> {
        &self.memory
    }

    #[inline]
    pub fn borrow_mut(&mut self) -> &mut SharedPointer<u8> {
        &mut self.memory
    }

    #[inline]
    pub fn clear_address_buffer(&mut self) {
        self.address_buffer.clear();
        self.filter.clear();
    }

    /// ## 惰性计算地址
    ///
    /// 物理地址直接返回
    ///
    /// 逻辑地址寻址时，先查看缓存，缓存中有则直接使用，没有才去访问页表计算物理地址
    pub fn address(
        &mut self,
        addr: u64,
        flag: u64,
        kpt: u64,
        upt: u64,
        rw: ReadWrite,
    ) -> Result<u64, AddressError> {
        let target = if flag.bit_get(FlagRegFlag::PagingEnabled) {
            if self.address_buffer.contains_key(&addr) {
                while let Some(mut val) = self.filter.pop() {
                    val.grow();
                    if val.lgaddr == addr {
                        val.increase();
                        self.filter.push(val.clone());
                    } else {
                        val.decrease();
                        self.filter.push(val.clone());
                    }
                }
                *self.address_buffer.get(&addr).unwrap()
            } else {
                let target = match self.calculate_addr(addr, kpt, upt, rw) {
                    Ok(addr) => addr,
                    Err(err) => match err {
                        CalcAddrError::OverSized => return Err(AddressError::OverSized(addr)),
                        CalcAddrError::Unreadable => return Err(AddressError::Unreadable),
                        CalcAddrError::Unwritable => return Err(AddressError::Unwritable),
                        CalcAddrError::Ineffective => return Err(AddressError::Ineffective),
                    },
                };
                if target < self.memory.size() as u64 {
                    self.address_buffer.insert(addr, target);
                    self.filter.push(UsageCounter::new(addr));
                    if self.address_buffer.len() > 128 {
                        while let Some(val) = self.filter.pop() {
                            if !val.is_newer() {
                                self.address_buffer.remove(&val.lgaddr);
                                break;
                            }
                            self.filter.push(val);
                        }
                    }
                }
                target
            }
        } else {
            addr
        };
        if target >= self.memory.size() as u64 {
            Err(AddressError::OverSized(target))
        } else if flag.bit_get(FlagRegFlag::Privilege) && (addr & (1 << 63)) == 0 {
            Err(AddressError::WrongPrivilege)
        } else {
            Ok(target)
        }
    }

    /// ## 查表寻址
    fn calculate_addr(
        &self,
        mut addr: u64,
        kpt: u64,
        upt: u64,
        rw: ReadWrite,
    ) -> Result<u64, CalcAddrError> {
        let userspace = addr.bit_get(FlagRegFlag::UserSpace);
        let offset = addr & 0x3fff;
        addr >>= 14;
        let entry_l1 = addr & 0x1f;
        addr >>= 9;
        let entry_l2 = addr & 0x1f;
        addr >>= 9;
        let entry_l3 = addr & 0x1f;
        addr >>= 9;
        let entry_l4 = addr & 0x1f;

        /* 四级页表寻址 */
        let table_l4 = if userspace { upt } else { kpt };
        let table_l4 = table_l4 - table_l4 & 0x3fff;
        let mut i = 0u64;
        let table_l4_len = loop {
            if self
                .memory
                .slice(table_l4 + i * 8, 8)
                .iter()
                .all(|x| *x == 0)
                || i == 1024
            {
                break i;
            }
            i += 1;
        };
        // 溢出检测
        if entry_l4 >= table_l4_len {
            return Err(CalcAddrError::OverSized);
        }
        // 取页表项
        let entry_l4_data = self.memory.slice(table_l4 + entry_l4 * 8, 8);
        let entry_l4_data = {
            let mut res = 0u64;
            for i in entry_l4_data.iter().rev() {
                res <<= 8;
                res |= *i as u64;
            }
            res
        };
        // 读写权限检测
        if !entry_l4_data.rwdetect(rw) {
            return match rw {
                ReadWrite::Read => Err(CalcAddrError::Unreadable),
                ReadWrite::Write => Err(CalcAddrError::Unwritable),
            };
        }
        // 有效性检测
        if !entry_l4_data.effdetect() {
            return Err(CalcAddrError::Ineffective);
        }
        // 大页直接返回
        if entry_l4_data.lpdetect() {
            return Ok(entry_l4_data - entry_l4_data
                & 0xfffffffffff
                    + offset
                    + (entry_l1 << 14)
                    + (entry_l2 << (14 + 9))
                    + (entry_l3 << (14 + 18)));
        }

        /* 三级页表寻址 */
        let table_l3 = entry_l4_data - entry_l4_data & 0x3fff;
        i = 0u64;
        let table_l3_len = loop {
            if self
                .memory
                .slice(table_l3 + i * 8, 8)
                .iter()
                .all(|x| *x == 0)
                || i == 1024
            {
                break i;
            }
            i += 1;
        };
        // 溢出检测
        if entry_l3 >= table_l3_len {
            return Err(CalcAddrError::OverSized);
        }
        // 取页表项
        let entry_l3_data = self.memory.slice(table_l3 + entry_l3 * 8, 8);
        let entry_l3_data = {
            let mut res = 0u64;
            for i in entry_l3_data.iter().rev() {
                res <<= 8;
                res |= *i as u64;
            }
            res
        };
        // 读写权限检测
        if !entry_l3_data.rwdetect(rw) {
            return match rw {
                ReadWrite::Read => Err(CalcAddrError::Unreadable),
                ReadWrite::Write => Err(CalcAddrError::Unwritable),
            };
        }
        // 有效性检测
        if !entry_l3_data.effdetect() {
            return Err(CalcAddrError::Ineffective);
        }
        // 大页直接返回
        if entry_l3_data.lpdetect() {
            return Ok(entry_l3_data - entry_l3_data
                & 0x3ffffffff + offset + (entry_l1 << 14) + (entry_l2 << (14 + 9)));
        }

        /* 二级页表寻址 */
        let table_l2 = entry_l3_data - entry_l3_data & 0x3fff;
        i = 0u64;
        let table_l2_len = loop {
            if self
                .memory
                .slice(table_l2 + i * 8, 8)
                .iter()
                .all(|x| *x == 0)
                || i == 1024
            {
                break i;
            }
            i += 1;
        };
        // 溢出检测
        if entry_l2 >= table_l2_len {
            return Err(CalcAddrError::OverSized);
        }
        // 取页表项
        let entry_l2_data = self.memory.slice(table_l2 + entry_l2 * 8, 8);
        let entry_l2_data = {
            let mut res = 0u64;
            for i in entry_l2_data.iter().rev() {
                res <<= 8;
                res |= *i as u64;
            }
            res
        };
        // 读写权限检测
        if !entry_l2_data.rwdetect(rw) {
            return match rw {
                ReadWrite::Read => Err(CalcAddrError::Unreadable),
                ReadWrite::Write => Err(CalcAddrError::Unwritable),
            };
        }
        // 有效性检测
        if !entry_l2_data.effdetect() {
            return Err(CalcAddrError::Ineffective);
        }
        // 大页直接返回
        if entry_l2_data.lpdetect() {
            return Ok(entry_l2_data - entry_l2_data & 0xffffff + offset + (entry_l1 << 14));
        }

        /* 一级页表寻址 */
        let table_l1 = entry_l2_data - entry_l2_data & 0x3fff;
        i = 0u64;
        let table_l1_len = loop {
            if self
                .memory
                .slice(table_l1 + i * 8, 8)
                .iter()
                .all(|x| *x == 0)
                || i == 1024
            {
                break i;
            }
            i += 1;
        };
        // 溢出检测
        if entry_l1 >= table_l1_len {
            return Err(CalcAddrError::OverSized);
        }
        // 取页表项
        let entry_l1_data = self.memory.slice(table_l1 + entry_l1 * 8, 8);
        let entry_l1_data = {
            let mut res = 0u64;
            for i in entry_l1_data.iter().rev() {
                res <<= 8;
                res |= *i as u64;
            }
            res
        };
        // 读写权限检测
        if !entry_l1_data.rwdetect(rw) {
            return match rw {
                ReadWrite::Read => Err(CalcAddrError::Unreadable),
                ReadWrite::Write => Err(CalcAddrError::Unwritable),
            };
        }
        // 有效性检测
        if !entry_l1_data.effdetect() {
            return Err(CalcAddrError::Ineffective);
        }
        Ok(entry_l2_data - entry_l2_data & 0x3fff + offset)
    }
}

enum PageEntryFlags {
    Effectivity = 0,
    LargePage = 1,
    Readability = 2,
    Writabilty = 3,
}

trait DetectPermission {
    /// ## 检测读写权限标志
    ///
    /// 返回false表示对应权限不满足
    fn rwdetect(&self, rw: ReadWrite) -> bool;

    /// ## 检测页或页表有效性
    ///
    /// 返回false表示页或页表无效
    fn effdetect(&self) -> bool;

    /// ## 检测此页表项是否指向大页
    fn lpdetect(&self) -> bool;
}

impl DetectPermission for u64 {
    #[inline]
    fn rwdetect(&self, rw: ReadWrite) -> bool {
        match rw {
            ReadWrite::Read => *self & PageEntryFlags::Readability as u64 != 0,
            ReadWrite::Write => *self & PageEntryFlags::Writabilty as u64 != 0,
        }
    }

    #[inline]
    fn effdetect(&self) -> bool {
        *self & PageEntryFlags::Effectivity as u64 != 0
    }

    fn lpdetect(&self) -> bool {
        *self & PageEntryFlags::LargePage as u64 != 0
    }
}

#[derive(Copy, Clone)]
pub enum ReadWrite {
    Read,
    Write,
}

enum CalcAddrError {
    OverSized,
    Unreadable,
    Unwritable,
    Ineffective,
}

#[derive(Debug, Clone, Ord, PartialEq, Eq)]
/// ## 使用计数
///
/// 用于记录某逻辑地址的使用次数，用以排序，并在address_buffer元素数量超过128时
/// 移除访问次数最小的地址
///
/// ### 新手保护
///
/// 刚刚加入到address_buffer中的元素可能会被立即移除，设置一个newer成员，在newer成员减为0
/// 以后才可以被移除
struct UsageCounter {
    lgaddr: u64,
    counter: isize,
    newer: u8,
}

impl UsageCounter {
    fn new(addr: u64) -> Self {
        Self {
            lgaddr: addr,
            counter: 0,
            newer: 16,
        }
    }

    #[inline]
    fn is_newer(&self) -> bool {
        self.newer != 0
    }

    #[inline]
    fn increase(&mut self) {
        self.counter += 1;
    }

    #[inline]
    fn decrease(&mut self) {
        self.counter -= 1;
    }

    #[inline]
    fn grow(&mut self) {
        if self.is_newer() {
            self.newer -= 1;
        }
    }
}

impl PartialOrd for UsageCounter {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.counter.partial_cmp(&other.counter)
    }
}
