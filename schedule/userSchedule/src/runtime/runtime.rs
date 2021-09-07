use std::{collections::VecDeque, sync::{Arc, Mutex}, task::{Context, Poll}};

use futures::{task::{ArcWake, waker_ref}};

use crate::kernel_task::{KernelThread, Task, TaskState};


//const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
//const MAX_THREADS: usize = 512;
// 运行时全局指针
pub static mut RUNTIME: usize = 0;

pub struct TaskMeta {
    // 线程和任务队列都要保存Task
    pub task: Arc<Task>,
    pub thread: Arc<Mutex<KernelThread>>,
    pub thread_id: usize,
}

impl TaskMeta {
    pub fn new(_task: Arc<Task>) -> Arc<Self> {
        Arc::new(TaskMeta {
            thread: _task.thread.clone(),
            task: _task.clone(),
            thread_id: _task.thread.lock().as_ref().unwrap().id.0,
        })
    }
    // 任务唤醒函数的封装。即设置任务状态为可运行Ready
    pub unsafe fn do_wake(self: Arc<Self>) {
        set_task_state(self.task.clone(), TaskState::Ready);
    }
}

/// 设置任务状态
pub fn set_task_state(task: Arc<Task>, state: TaskState) {
    *task.state.lock().unwrap() = state;
    
}
// 实现此trait才可以调用 waker_ref 创建waker
impl ArcWake for TaskMeta {
    fn wake_by_ref(arc_tm: &Arc<Self>) {
        unsafe {
            arc_tm.clone().do_wake();
        }
    }
    
}

pub struct Runtime {
    pub tasks: VecDeque<Arc<TaskMeta>>,
    pub current: Arc<TaskMeta>,
}

impl Runtime {
    /// 封装Task为任务队列中的TaskMeta，并插入任务队列
    pub fn add_task(&mut self, task: Arc<Task>){
        let metadata = TaskMeta::new(task);
        self.tasks.push_back(metadata);
    }

    pub fn new() -> Self {
        // 创建第0个task插入
        let idle_task = Task::idle(||{});
        let idle_mt = TaskMeta::new(idle_task.clone());
        let mut deque = VecDeque::new();
        deque.push_back(idle_mt.clone());
        let rt = Runtime {
            tasks: deque,
            current: idle_mt.clone(),
        };

        rt
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }
    /// 启动运行时
    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        println!("runtime run finish");
        std::process::exit(0);
    }
    /// 查找下一个可以执行的任务并执行
    fn t_yield(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        if self.tasks.front().unwrap().thread_id == 1 {
            let prev_task = self.tasks.pop_front().unwrap();
            self.tasks.push_back(prev_task);
        }

        let next_task = self.tasks.pop_front().unwrap();
        if next_task.thread_id == 1 {
            return false;
        }

        // 注册waker
        let waker = waker_ref(&next_task);
        let cx = &mut Context::from_waker(&*waker);
        // 执行任务
        self.current = next_task.clone();
        let res = next_task.task.future.try_lock().unwrap().as_mut().poll(cx);
        
        if let Poll::Pending = res {
            // 如果任务未完成，重新插入队列
            self.tasks.push_back(next_task);
        }

        !self.is_empty()
    }
    /// 任务队列是否为空
    pub fn is_empty(&self) -> bool {
        self.tasks.len() <= 1
    }
}



pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    };
}

#[naked]
#[allow(unsupported_naked_functions)]
#[inline(never)]
unsafe fn switch() {
    llvm_asm!("
        mov     %rsp, 0x00(%rdi)
        mov     %r15, 0x08(%rdi)
        mov     %r14, 0x10(%rdi)
        mov     %r13, 0x18(%rdi)
        mov     %r12, 0x20(%rdi)
        mov     %rbx, 0x28(%rdi)
        mov     %rbp, 0x30(%rdi)

        mov     0x00(%rsi), %rsp
        mov     0x08(%rsi), %r15
        mov     0x10(%rsi), %r14
        mov     0x18(%rsi), %r13
        mov     0x20(%rsi), %r12
        mov     0x28(%rsi), %rbx
        mov     0x30(%rsi), %rbp
        "
    );
}