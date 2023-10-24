/// flag寄存器的标志位
///
/// > 详见vrisc架构文档
pub enum FlagRegFlag {
    Zero = 0,
    Symbol = 1,
    Overflow = 2,
    Equal = 3,
    Higher = 4,
    Lower = 5,
    Bigger = 6,
    Smaller = 7,
    InterruptEnabled = 8,
    PagingEnabled = 9,
    Privilege = 10,

    UserSpace = 63,
}

/// 指令的条件码
///
/// > 详见vrisc架构文档
pub enum ConditionCode {
    None = 0,
    Zero = 1,
    Signed = 2,
    Overflow = 3,
    Equal = 4,
    NonEqual = 5,
    Higher = 6,
    Lower = 7,
    NonHigher = 8,
    NonLower = 9,
    Bigger = 10,
    Smaller = 11,
    NonBigger = 12,
    NonSmaller = 13,
}

impl ConditionCode {
    pub fn new(cond: u8) -> Self {
        match cond {
            0 => ConditionCode::None,
            1 => ConditionCode::Zero,
            2 => ConditionCode::Signed,
            3 => ConditionCode::Overflow,
            4 => ConditionCode::Equal,
            5 => ConditionCode::NonEqual,
            6 => ConditionCode::Higher,
            7 => ConditionCode::Lower,
            8 => ConditionCode::NonHigher,
            9 => ConditionCode::NonLower,
            10 => ConditionCode::Bigger,
            11 => ConditionCode::Smaller,
            12 => ConditionCode::NonBigger,
            13 => ConditionCode::NonSmaller,
            _ => ConditionCode::None,
        }
    }
}

/// vrisc寄存器
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Registers {
    pub x: [u64; 16],
    pub ip: u64,
    pub flag: u64,
    pub ivt: u64,
    pub kpt: u64,
    pub upt: u64,
    pub scp: u64,
    pub imsg: u64,
    pub ipdump: u64,
    pub flagdump: u64,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            x: [0; 16],
            ip: 0,
            flag: 0,
            ivt: 0,
            kpt: 0,
            upt: 0,
            scp: 0,
            imsg: 0,
            ipdump: 0,
            flagdump: 0,
        }
    }

    pub fn reset(&mut self) {
        self.x.copy_from_slice(&[0; 16]);
        self.ip = 0;
        self.flag = 0;
        self.ivt = 0;
        self.kpt = 0;
        self.upt = 0;
        self.scp = 0;
        self.imsg = 0;
        self.ipdump = 0;
        self.flagdump = 0;
    }
}
