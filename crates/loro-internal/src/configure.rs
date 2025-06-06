use fxhash::FxHashSet;
use loro_common::ContainerID;

pub use crate::container::richtext::config::{StyleConfig, StyleConfigMap};
use crate::LoroDoc;
use std::sync::atomic::{AtomicBool, AtomicI64};
use std::sync::RwLock;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Configure {
    pub(crate) text_style_config: Arc<RwLock<StyleConfigMap>>,
    record_timestamp: Arc<AtomicBool>,
    pub(crate) merge_interval_in_s: Arc<AtomicI64>,
    pub(crate) editable_detached_mode: Arc<AtomicBool>,
    pub(crate) deleted_root_containers: Arc<Mutex<FxHashSet<ContainerID>>>,
    pub(crate) hide_empty_root_containers: Arc<AtomicBool>,
}

impl LoroDoc {
    pub(crate) fn set_config(&self, config: &Configure) {
        self.config_text_style(config.text_style_config.read().unwrap().clone());
        self.set_record_timestamp(config.record_timestamp());
        self.set_change_merge_interval(config.merge_interval());
        self.set_detached_editing(config.detached_editing());
    }
}

impl Default for Configure {
    fn default() -> Self {
        Self {
            text_style_config: Arc::new(RwLock::new(StyleConfigMap::default_rich_text_config())),
            record_timestamp: Arc::new(AtomicBool::new(false)),
            editable_detached_mode: Arc::new(AtomicBool::new(false)),
            merge_interval_in_s: Arc::new(AtomicI64::new(1000)),
            deleted_root_containers: Arc::new(Mutex::new(Default::default())),
            hide_empty_root_containers: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Configure {
    pub fn fork(&self) -> Self {
        Self {
            text_style_config: Arc::new(RwLock::new(
                self.text_style_config.read().unwrap().clone(),
            )),
            record_timestamp: Arc::new(AtomicBool::new(
                self.record_timestamp
                    .load(std::sync::atomic::Ordering::Relaxed),
            )),
            merge_interval_in_s: Arc::new(AtomicI64::new(
                self.merge_interval_in_s
                    .load(std::sync::atomic::Ordering::Relaxed),
            )),
            editable_detached_mode: Arc::new(AtomicBool::new(
                self.editable_detached_mode
                    .load(std::sync::atomic::Ordering::Relaxed),
            )),
            deleted_root_containers: Arc::new(Mutex::new(
                self.deleted_root_containers.lock().unwrap().clone(),
            )),
            hide_empty_root_containers: Arc::new(AtomicBool::new(
                self.hide_empty_root_containers
                    .load(std::sync::atomic::Ordering::Relaxed),
            )),
        }
    }

    pub fn text_style_config(&self) -> &Arc<RwLock<StyleConfigMap>> {
        &self.text_style_config
    }

    pub fn record_timestamp(&self) -> bool {
        self.record_timestamp
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_record_timestamp(&self, record: bool) {
        self.record_timestamp
            .store(record, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn detached_editing(&self) -> bool {
        self.editable_detached_mode
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_detached_editing(&self, mode: bool) {
        self.editable_detached_mode
            .store(mode, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn merge_interval(&self) -> i64 {
        self.merge_interval_in_s
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_merge_interval(&self, interval: i64) {
        self.merge_interval_in_s
            .store(interval, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_hide_empty_root_containers(&self, hide: bool) {
        self.hide_empty_root_containers
            .store(hide, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct DefaultRandom;

#[cfg(test)]
use std::sync::atomic::AtomicU64;
#[cfg(test)]
static mut TEST_RANDOM: AtomicU64 = AtomicU64::new(0);

impl SecureRandomGenerator for DefaultRandom {
    fn fill_byte(&self, dest: &mut [u8]) {
        #[cfg(not(test))]
        getrandom::getrandom(dest).unwrap();

        #[cfg(test)]
        // SAFETY: this is only used in test
        unsafe {
            #[allow(static_mut_refs)]
            let bytes = TEST_RANDOM.fetch_add(1, std::sync::atomic::Ordering::Release);
            dest.copy_from_slice(&bytes.to_le_bytes());
        }
    }
}

pub trait SecureRandomGenerator: Send + Sync {
    fn fill_byte(&self, dest: &mut [u8]);
    fn next_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_byte(&mut buf);
        u64::from_le_bytes(buf)
    }

    fn next_u32(&self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_byte(&mut buf);
        u32::from_le_bytes(buf)
    }

    fn next_i64(&self) -> i64 {
        let mut buf = [0u8; 8];
        self.fill_byte(&mut buf);
        i64::from_le_bytes(buf)
    }

    fn next_i32(&self) -> i32 {
        let mut buf = [0u8; 4];
        self.fill_byte(&mut buf);
        i32::from_le_bytes(buf)
    }
}
