use std::{collections::VecDeque, sync::{Arc, Mutex}};

use crate::task::{Task};

pub static mut SCHED: usize = 0;


/// 调度器，维护一个任务队列
pub struct Scheduler {
    pub list: Arc<Mutex<VecDeque<Task>>>,
}

#[allow(dead_code)]
impl Scheduler {
    /// 创建调度器
    pub fn new() -> Self {
        Self {
            list: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// 初始化调度器指针，必须在 new 之后调用
    pub fn init(&self) {
        unsafe {
            let s_ptr: *const Scheduler = self;
            SCHED = s_ptr as usize;
        }
    }

    /// 队首任务引用
    //pub fn peek_as_ref(&self) -> Option<&Task> {
    //    let list = self.list.lock().unwrap().front();
    //    list
    //}

    /// 弹出队首任务
    pub fn pop(&mut self) -> Option<Task> {
        self.list.lock().unwrap().pop_front()
    }

    /// 插入任务到队列尾
    pub fn push(&mut self, task: Task) {
        self.list.lock().unwrap().push_back(task);
    }

}