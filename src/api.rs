pub mod interface {
    pub use crate::utils::memory::Memory;
    pub use crate::utils::shared::Addressable;
    pub use crate::utils::shared::SharedPointer;
    pub use crate::vrisc::vcore::dma::DMADevice;
    pub use crate::vrisc::vcore::dma::DMAObject;
    pub use crate::vrisc::vcore::iocontroller::IODevice;
}

pub mod device {
    /// # 字符设备
    ///
    /// 以虚拟键盘设备为例：
    ///
    /// ```rust
    /// use vcore::device::char::CharacterDevice;
    ///
    /// let dev = CharacterDevice::new();
    /// let dev = Arc::new(RwLock::new(dev));
    /// let devrec = Arc::clone(&dev);
    /// let (tx, rx) = mpsc::channel();
    ///
    /// thread::spawn(move || {
    ///     loop {
    ///         let kc2 = get_key_code();   // 通过虚拟键盘的ui界面的api获取按键的键盘码
    ///         tx.send(kc2).unwrap();
    ///     }
    /// });
    ///
    /// // 接收设置键盘指示灯状态的线程
    /// thread::spawn(move || {
    ///     loop {
    ///         if let Some(data) = { devrec.write().unwrap().output() } {
    ///             for byte in data {
    ///                 match byte {
    ///                     0 => break,
    ///                     ~ => set_led_status(~, ~),
    ///                     ...
    ///                     _ => (),
    ///                 }
    ///             }
    ///         }
    ///     }
    /// });
    ///
    /// // 向vcore发送键盘事件的循环
    /// loop {
    ///     let kc2 = rx.recv().unwrap();
    ///     let kc2 = vec![kc2,];
    ///     dev.write().unwrap().input(&kc2);
    /// }
    /// ```
    pub mod char {
        pub use crate::vrisc::vcore::iocontroller::CharacterDevice;
    }

    /// # 块设备
    ///
    /// 所有块设备都通过io和dma与vcore交换信息。
    ///
    /// 如显示器和磁盘等，需要传输的画面和磁盘数据块直接写入dma的内存。
    ///
    /// `块设备`抽象层在使用`BlockDevice::new()`后自动连接io端口，而dma内存区域需要
    /// 驱动程序与设备通信确定并连接。
    pub mod block {
        pub use crate::vrisc::vcore::iocontroller::BlockDevice;
    }
}
