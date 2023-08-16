use num_traits::Num;

pub trait RadixParse<T: Num> {
    fn rdxparse(&self) -> Result<T, T::FromStrRadixErr>;
}

impl<U: ToString, T: Num> RadixParse<T> for U {
    fn rdxparse(&self) -> Result<T, T::FromStrRadixErr> {
        let mut s = self.to_string();
        let radix = if s.starts_with("0x") {
            s.remove(0);
            s.remove(0);
            16u8
        } else if s.starts_with("0o") {
            s.remove(0);
            s.remove(0);
            8
        } else if s.starts_with("0b") {
            s.remove(0);
            s.remove(0);
            2
        } else {
            10
        };
        T::from_str_radix(&s, radix as u32)
    }
}
