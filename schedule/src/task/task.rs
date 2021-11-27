use std::{future::Future, pin::Pin, sync::{ Mutex, atomic::AtomicUsize}, task::Poll};

use crate::scheduler::{SCHED, Scheduler};

pub struct TaskId(pub usize);

impl TaskId {
    fn get_id() -> TaskId {
        // static变量只初始化一次，内核线程中，应使用 AtomicUsize 确保 COUNTER 线程安全
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        TaskId(id)
    }
}

/// 协程：保存在进程的地址空间中，进程与线程不需要有保存协程的列表，在对应的进程地址空间中，执行器可以直接执行协程
///
/// 优先级：0, 1, 2, 3, 默认情况下设置为 1。
pub struct Task {
    pub id: TaskId,
    pub future: Mutex<Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>>,
    pub process: usize,
    pub priority: usize,
}

#[allow(dead_code)]
impl Task {
    /// 创建协程 + 插入任务队列
    pub fn new(
        f: impl FnOnce() + 'static + Send + Sync,
        pid: usize,
    ){

       let task_future = async {
           f();
       };

       let task = Task {
            id: TaskId::get_id(),
            future: Mutex::new(Box::pin(task_future)),
            process: pid,
            priority: 1,
        };

        // 取得调度器，插入任务队列
        unsafe {
            let s_ptr = SCHED as *mut Scheduler;
            (*s_ptr).push(task);
        }
    }

    /// 创建协程 + 插入任务队列 + 指定优先级
    pub fn new_with_p(
        f: impl FnOnce() + 'static + Send + Sync,
        pri: usize,
        pid: usize,
    ){

       let task_future = async {
           f();
       };

       let task = Task {
            id: TaskId::get_id(),
            future: Mutex::new(Box::pin(task_future)),
            process: pid,
            priority: pri,
        };

        // 取得调度器，插入任务队列
        unsafe {
            let s_ptr = SCHED as *mut Scheduler;
            (*s_ptr).push(task);
        }
    }

    // 创建一个协程计算fbnc数列，每进行一次加法会主动退出
    pub fn new_fnbc(cnt: usize, pid: usize) {
        let task = Task {
            id: TaskId::get_id(),
            future: Mutex::new(Box::pin(FibonacciFuture::new(cnt))),
            process: pid,
            priority: 1,
        };

        // 取得调度器，插入任务队列
        unsafe {
            let s_ptr = SCHED as *mut Scheduler;
            (*s_ptr).push(task);
        }
    }

    
}



struct FibonacciFuture {
    a: usize,
    b: usize,
    cnt: usize,
}

impl FibonacciFuture {
    fn new(cnt: usize) -> FibonacciFuture {
        FibonacciFuture {
            a: 0,
            b: 1,
            cnt,
        }
    }
}

impl Future for FibonacciFuture {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.b >= self.cnt {
            println!("  fbnc协程,  {} >= {}", self.b, self.cnt);
            return Poll::Ready(());
        }

        println!("  fbnc协程, current = {}", self.b);

        let c = self.b;
        self.b = self.a + self.b;
        self.a = c;

        Poll::Pending
    }
}