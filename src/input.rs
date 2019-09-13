use log::info;
use url::Url;

use crate::error::Result;

pub trait Input {
    fn open(&self) -> Result<()>;
    fn read(&self) -> Result<()>;
    fn close(&self) -> Result<()>;
}

pub struct InputUDP {
    url: Url,

    /// a.k.a. circular buffer
    fifo_sz: usize,
}

impl InputUDP {
    pub fn new(url: Url) -> InputUDP {
        InputUDP {
            url,
            fifo_sz: 5 * 1000,
        }
    }
}

impl Input for InputUDP {
    fn open(&self) -> Result<()> {
        Ok(())
    }
    fn read(&self) -> Result<()> {
        info!("[<] {} {}", self.url, self.fifo_sz);
        Ok(())
    }
    fn close(&self) -> Result<()> {
        Ok(())
    }
}

struct InputFile {}

impl Input for InputFile {
    fn open(&self) -> Result<()> {
        Ok(())
    }
    fn read(&self) -> Result<()> {
        Ok(())
    }
    fn close(&self) -> Result<()> {
        Ok(())
    }
}
