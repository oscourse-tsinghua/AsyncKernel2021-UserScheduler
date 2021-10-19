use std::{pin::Pin, sync::Arc, task::{Context, Poll}};

use futures::task::{ArcWake, waker_ref};

use crate::{runtime::{RUNTIME, Runtime}, scheduler::{SCHED, Scheduler}};


/// 线程栈大小
#[allow(dead_code)]
const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum State {
    Available,
    Running,
    Ready,
}


#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct ThreadContext {
    pub rsp: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
}


#[allow(dead_code)]
#[derive(Clone)]
pub struct MyThread {
    pub id: usize,
    pub stack: Pin<Box<Vec<u8>>>,
    pub ctx: Pin<Box<ThreadContext>>,
    pub state: State,
}

#[allow(dead_code)]
impl MyThread {
    pub fn new(id: usize) -> Self {
        MyThread {
            id,
            stack: Box::pin(vec![0_u8; DEFAULT_STACK_SIZE]),
            ctx: Box::pin(ThreadContext::default()),
            state: State::Available,
        }
    }
    // waker的具体实现：将线程设置为Ready，使得线程执行的协程可以继续执行
    pub fn do_wake() {

    }

    // 设置线程为指定状态
    pub fn set_state() {

    }
}

impl ArcWake for MyThread {
    fn wake_by_ref(_t: &Arc<Self>) {
        
    }
}

pub fn thread_main() {
    loop {
        // 取出任务队列的头部引用
        // todo()! 判断地址空间
        let s_ptr;
        let len;
        unsafe {
            s_ptr = SCHED as *mut Scheduler;
            len = (*s_ptr).list.lock().unwrap().len();
        }
        // 任务队列为空，线程退出
        if len == 0 { break; }

        // todo()! 地址空间判断

        // 从调度器取出任务
        let mut task;
        unsafe {
            task = (*s_ptr).pop();
        }
        // 理想的做法是waker通过参数传入
        let arc_t;
        unsafe {
            let r_ptr = RUNTIME.with(|r| *r.borrow()) as *mut Runtime;
            let pos = (*r_ptr).current;
            let tid = (*r_ptr).threads[pos].id;
            println!("线程 {}，从调度器取走协程执行", tid);
            arc_t = Arc::new((*r_ptr).threads[(*r_ptr).current].clone());
        }
        let waker = waker_ref(&arc_t);
        let cx = &mut Context::from_waker(&*waker);

        let ret = task.as_mut().unwrap().future.lock().unwrap().as_mut().poll(cx);
        // 任务未完成，插回任务队列
        if let Poll::Pending = ret {
            println!("协程主动让出，插回调度器");
            unsafe {
                (*s_ptr).push(task.unwrap());
            }
        }
    }
}

#[naked]
#[inline(never)]
#[allow(unsupported_naked_functions, dead_code)]
pub unsafe fn switch() {
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