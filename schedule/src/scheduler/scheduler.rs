use std::{collections::VecDeque, sync::{Arc, Mutex}};

use crate::task::{Task};

pub static mut SCHED: usize = 0;


/// 调度器，维护一个任务队列
/// 
/// 二维数组保存任务队列，第一维是优先级，默认情况下弹出当前队列中优先级最高的任务。
pub struct Scheduler {
    pub list: Arc<Mutex<Vec<VecDeque<Task>>>>,
    pub task_nums: usize,
}

#[allow(dead_code)]
impl Scheduler {
    /// 创建调度器
    pub fn new() -> Self {
        let list = (0..4).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<Task>>>();
        Self {
            list: Arc::new(Mutex::new(list)),
            task_nums: 0,
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

    /// 弹出最高优先级任务
    pub fn pop(&mut self) -> Option<Task> {
        let ret = self.list.lock().map(|mut l| {
            for i in 0..4 {
                if l[i].len() != 0 {
                    return l[i].pop_front();
                }
            }
            return None;
        }).unwrap();
        self.task_nums -= 1;

        ret
    }

    /// 插入任务到队列尾
    #[allow(unused_must_use)]
    pub fn push(&mut self, task: Task) {
        self.list.lock().map(|mut l| {
            let pri = task.priority;
            l[pri].push_back(task);
        });

        self.task_nums += 1;
    }

}