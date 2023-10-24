/// ## 惰性寻址系统
///
/// hot_ip时栈上储存的ip寄存器的寻址后值，只有这个值每运行一次指令改变一次。
///
/// 此ip不重新寻址，因为在一个页内，物理地址与线性地址一一对应。
///
/// 满足如下条件时，hot_ip同步至core.regs.ip：
/// * 产生转移：需要将ip（中断转移还需flag）转存至dump寄存器
/// 满足如下条件时，重新为hot_ip寻址：
/// * 产生转移：转移很可能导致ip不在此页中
/// * 遇到最小页边界：此时地址大概率不在同一页中。“大概率”指有时分页会有大页，在大页中的较小页边界两侧的内存
///         都在同一页中，但是由于最小页有16KB，遇到最小页边界的概率也不大，判断一个最小页边界是否是此页
///         的边界会消耗更多时间（这得从顶级页表开始一级一级地查才能查到）。因此只要遇到最小页边界就更新，
///         不要判断是否确实是页边界。
///
/// 在此顺便说明，core.ip_increment是自core.regs.ip被同步以来的总increment
pub struct LazyAddress {
    pub hot_ip: u64,
    pub had_run_inst: bool,
    pub crossed_page: bool,
}

impl LazyAddress {
    pub fn new() -> Self {
        Self {
            hot_ip: 0,
            had_run_inst: false,
            crossed_page: false,
        }
    }
}
