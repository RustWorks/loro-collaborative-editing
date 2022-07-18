use crate::{change::ChangeMergeCfg, log_store::GcConfig, Timestamp};
use ring::rand::{SecureRandom, SystemRandom};

pub struct Configure {
    pub change: ChangeMergeCfg,
    pub gc: GcConfig,
    pub(crate) get_time: fn() -> Timestamp,
    pub(crate) rand: Box<dyn SecureRandomGenerator>,
}

pub trait SecureRandomGenerator {
    fn fill_byte(&mut self, dest: &mut [u8]);
    fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_byte(&mut buf);
        u64::from_le_bytes(buf)
    }

    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_byte(&mut buf);
        u32::from_le_bytes(buf)
    }

    fn next_i64(&mut self) -> i64 {
        let mut buf = [0u8; 8];
        self.fill_byte(&mut buf);
        i64::from_le_bytes(buf)
    }

    fn next_i32(&mut self) -> i32 {
        let mut buf = [0u8; 4];
        self.fill_byte(&mut buf);
        i32::from_le_bytes(buf)
    }
}

impl SecureRandomGenerator for SystemRandom {
    fn fill_byte(&mut self, dest: &mut [u8]) {
        self.fill(dest).unwrap();
    }
}

impl Default for Configure {
    fn default() -> Self {
        Self {
            change: ChangeMergeCfg::default(),
            gc: GcConfig::default(),
            get_time: || 0,
            rand: Box::new(SystemRandom::new()),
        }
    }
}