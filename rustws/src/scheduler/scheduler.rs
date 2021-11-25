use std::{collections::VecDeque, sync::{Arc, Mutex, RwLock}};

use futures::SinkExt;

use crate::task::{Task};

pub static mut SCHED: usize = 0;


/// 调度器，维护一个任务队列
/// 
/// 二维数组保存任务队列，第一维是优先级，默认情况下弹出当前队列中优先级最高的任务。
pub struct Scheduler {
    pub list: Arc<Mutex<Vec<VecDeque<Task>>>>,
    pub task_nums: Arc<RwLock<usize>>,
    pub val: Arc<RwLock<usize>>,
}

const LEVEL_NUM: usize = 10;

#[allow(dead_code)]
impl Scheduler {
    /// 创建调度器
    pub fn new() -> Self {
        let list = (0..LEVEL_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<Task>>>();
        Self {
            list: Arc::new(Mutex::new(list)),
            task_nums: Arc::new(RwLock::new(0)),
            val: Arc::new(RwLock::new(0)),
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
    #[allow(unused_must_use)]
    pub fn pop(&mut self) -> Option<Task> {
        self.val.write().map(|mut v| *v += 1);

        let ret = self.list.lock().map(|mut l| {
            let cnt = *self.task_nums.read().unwrap();
            if cnt == 0 { return None; }

            let v = *self.val.read().unwrap() % 100;

            let mut skip = false;
            for i in 0..LEVEL_NUM {

                if cnt == l[i].len() || (skip && l[i].len() > 0) || (l[i].len() > 0 && v == 0 ) {
                    // 如果只有一个队列有协程 || 之前已经跳过过一次
                    self.task_nums.write().map(|mut n| *n -= 1);
                    return l[i].pop_front();
                } else if l[i].len() >= 5 || (skip && l[i].len() > 0) || (l[i].len() > 0 && v == 0 ) {
                    // 如果有多个队列有协程，如果最低优先级
                    self.task_nums.write().map(|mut n| *n -= 1);
                    return l[i].pop_front();
                } else {
                    skip = true;
                }

            }
            return None;
        }).unwrap();
        
        ret
    }

    /// 插入任务到队列尾
    #[allow(unused_must_use)]
    pub fn push(&mut self, task: Task) {
        self.list.lock().map(|mut l| {
            let pri = task.priority;
            l[pri].push_back(task);
            self.task_nums.write().map(|mut n| *n += 1);
        });
    }

    #[allow(unused_must_use)]
    pub fn insert(&mut self, task: Task) {
        self.list.lock().map(|mut l| {
            if l[2].len() < 9 {
                l[2].push_back(task);
            } else {
                l[2].insert(9, task);
            }
            self.task_nums.write().map(|mut n| *n += 1);
        });
    }

}