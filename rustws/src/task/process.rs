use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};


pub struct ProcessId(usize);

#[allow(dead_code)]
impl ProcessId {
    pub fn get_id() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

        ProcessId(id)
    }
}

/// 进程控制块
#[allow(dead_code)]
pub struct Process {
    pid: ProcessId,

}

#[allow(dead_code)]
impl Process {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pid: ProcessId::get_id(),
        })
    }
}