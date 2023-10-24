#[derive(Debug, Clone, Copy)]
pub enum InterruptId {
    NI = 0,
    InaccessibleAddress = 1,
    Device = 2,
    Clock = 3,
    InvalidInstruction = 4,
    WrongPrivilege = 5,
    InaccessibleIOPort = 6,
    PageOrTableUnreadable = 7,
    PageOrTableUnwritable = 8,
    DeviceCommunication = 9,
}

impl InterruptId {
    /// 只要不在InterruptId中，都会返回NI（Not a Interrupt）
    pub fn generate(id: u8) -> Self {
        match id {
            1 => InterruptId::InaccessibleAddress,
            2 => InterruptId::Device,
            3 => InterruptId::Clock,
            4 => InterruptId::InvalidInstruction,
            5 => InterruptId::WrongPrivilege,
            6 => InterruptId::InaccessibleIOPort,
            7 => InterruptId::PageOrTableUnreadable,
            8 => InterruptId::PageOrTableUnwritable,
            9 => InterruptId::DeviceCommunication,
            _ => InterruptId::NI,
        }
    }
}

/// 中断控制器
///
/// 负责处理中断
pub struct InterruptController {
    intflag: bool,
    /// 在intflag为`true`时使用
    intid: InterruptId,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            intflag: false,
            intid: InterruptId::NI,
        }
    }

    pub fn interrupt(&mut self, intid: InterruptId) {
        self.intflag = true;
        self.intid = intid;
    }

    pub fn interrupted(&self) -> Option<InterruptId> {
        if self.intflag {
            Some(self.intid)
        } else {
            None
        }
    }

    pub fn reset_intflag(&mut self) {
        self.intflag = false;
    }

    pub fn reset(&mut self) {
        self.intflag = false;
        self.intid = InterruptId::NI;
    }
}
