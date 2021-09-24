/// Possible character sets
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum CodeTable {
    USA,
    Latin2,
}

impl CodeTable {
    /// Returns the byte representation of the esc/pos command
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            CodeTable::USA => vec![0x00],
            CodeTable::Latin2 => vec![0x02],
        }
    }
}
