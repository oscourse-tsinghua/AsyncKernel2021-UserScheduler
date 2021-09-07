use std::{sync::{Arc, Mutex}};

use super::Task;
use crate::runtime::{Runtime, RUNTIME};

pub struct ThreadId(pub usize);

impl ThreadId {
    fn get_id() -> ThreadId {
        // static变量只初始化一次，内核线程中，应使用 AtomicUsize 确保 COUNTER 线程安全
        static mut COUNTER: usize = 0;
        unsafe {
            COUNTER += 1;
            let id = COUNTER;
            ThreadId(id)
        }
        
    }  
}


#[allow(non_camel_case_types)]
pub struct KernelThread {
    pub id: ThreadId,
    // 访问控制？
    pub tasks: Vec<Arc<Task>>,
    pub ctx: ThreadContext,
}

impl KernelThread {
    /// 线程创建接口，传入的函数作为主协程，插入到线程和任务队列中
    pub fn new(
        f: impl FnOnce() + 'static + Send + Sync
    ) -> Arc<Mutex<Self>> {
        // 缺省主协程，创建线程
        let thread = Arc::new(Mutex::new(KernelThread {
            id: ThreadId::get_id(),
            tasks: vec![],
            ctx: ThreadContext::default(),
        }));
        
        // 创建主协程插入
        let task = Task::creat_task(f, thread.clone());
        thread.lock().as_mut().unwrap().add_task(task.clone());
        
        // 插入调度队列
        unsafe {
            let rt_ptr = RUNTIME as *mut Runtime;
            (*rt_ptr).add_task(task);
        }

        thread
    }

    pub fn add_task(&mut self, task: Arc<Task>) {
        let len = self.tasks.len();
        self.tasks.insert(len, task);
    }

    pub fn idle_thread() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(KernelThread {
            id: ThreadId::get_id(),
            tasks: vec![],
            ctx: ThreadContext::default(),
        }))
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}