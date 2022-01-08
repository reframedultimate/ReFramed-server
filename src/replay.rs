use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static!{
    static ref REPLAY_MANAGER: Mutex<ReplayManager> = Mutex::new(ReplayManager::new());
}

pub struct ReplayManager {
};

impl ReplayManager {
    pub fn get() -> &'static Mutex<Self> {
        &REPLAY_MANAGER
    }

    pub fn new() -> Self {
        Self {
        }
    }
}

