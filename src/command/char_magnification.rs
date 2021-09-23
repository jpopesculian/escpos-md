use crate::error::{Error, Result};

#[derive(Clone, Debug, Copy)]
pub struct CharMagnification {
    width: u8,
    height: u8,
}

impl CharMagnification {
    pub fn new(width: u8, height: u8) -> Result<Self> {
        Self::check_param(width)?;
        Self::check_param(height)?;
        Ok(CharMagnification { width, height })
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn check_param(param: u8) -> Result<()> {
        // 0 < param <= 8
        if param < 1 || param > 8 {
            Err(Error::InvalidCharMagnification)
        } else {
            Ok(())
        }
    }

    pub fn to_byte(&self) -> u8 {
        (self.width - 1) | ((self.height - 1) << 4)
    }
}

impl Default for CharMagnification {
    fn default() -> Self {
        CharMagnification {
            width: 1,
            height: 1,
        }
    }
}
