use std::{cell::RefCell, sync::atomic::AtomicUsize};

use crate::{NOYIFY, my_thread::*, scheduler::{SCHED, Scheduler}};


// 每一个物理核拥有运行一个独立的Runtime
thread_local! {pub static RUNTIME: RefCell<usize> = RefCell::new(0);}

pub struct CoreId(pub usize);

impl CoreId {
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self(id)
    }
}

pub struct Runtime {
    pub core_id: CoreId,
    pub threads: Vec<MyThread>,
    pub blocked: Vec<MyThread>,
    pub current: usize,
}

impl Runtime {
    pub fn new() -> Self {
        // 主线程，用于在上下文切换时保存主程序执行流程
        // 区别于使用 spawn 创建的线程，后者执行的是 thread_main 函数
        let mut main_thread = MyThread::new(0);
        main_thread.state = State::Running;

        Self {
            core_id: CoreId::new(),
            threads: vec![main_thread],
            blocked: vec![],
            current: 0,
        }
    }

    pub fn init(&self) {
        let r_ptr: *const Runtime = self;
        RUNTIME.with(move |r|r.replace(r_ptr as usize));
    }

    pub fn run(&mut self) {

        loop {
            // 获取调度器
            let s_ptr;
            // 得到任务队列长度
            let tasks_len;
            unsafe {
                s_ptr = SCHED as *mut Scheduler;
                tasks_len = *(*s_ptr).task_nums.read().unwrap();
            }
            
            if self.threads.len() > 1 {
                // 如果当前进程存在可以执行的线程，一般来自于从阻塞队列恢复的线程，或是新创建的线程
                //println!("core: {}, 存在就绪线程", self.core_id.0);
                self.t_yield();
            } else if tasks_len != 0 {
                // 如果线程列表为空，但任务队列不空，创建一个线程
                //println!("core: {}, 就绪线程数 == 0，创建线程绑定协程执行", self.core_id.0);
                self.spawn(thread_main);
            } else if self.blocked.len() == 0 {
                // 没有需要运行的协程，运行时结束
                //break ;
            } else {
                // 存在阻塞线程，查看 NOTIFY
                unsafe {
                    if NOYIFY == 1 { self.notify(); }
                }
            }
        }

        //std::process::exit(0);
    }

    /// 回收线程控制块，并重新启动运行时执行线程
    pub fn t_return(&mut self) {

        println!("core: {}, 协程执行结束，退出并删除线程: {}", self.core_id.0, self.threads[self.current].id);
        let delete_pos = self.current;
        let pos = 0;
        self.threads[pos].state = State::Ready;
        self.current = 0;
        
        let temp_ctx = &mut ThreadContext::default();
        
        // 删除当前线程
        self.threads.remove(delete_pos);
        // 切换到线程0
        unsafe {
            let old: *mut ThreadContext = temp_ctx;
            let new: *const ThreadContext = self.threads[pos].ctx.as_ref().get_ref();
            llvm_asm!(
                "mov $0, %rdi
                 mov $1, %rsi"::"r"(old), "r"(new)
            );
            switch();
        }
    }

    // 寻找可以执行的线程进行切换
    pub fn t_yield(&mut self) {
        // 寻找下一个可以执行的线程
        let mut pos = 1;
        while pos < self.threads.len() {
            if self.threads[pos].state == State::Ready {
                break;
            }
            pos += 1;
        }
        // 没有可以继续执行的线程
        if pos == self.threads.len() { return ; }

        //let sp_val = self.threads[pos].ctx.rsp;
        //let mask = (1 << 28) - 1;
        //let tid = (sp_val & mask) >> 20;
        println!("core: {}, 线程: {}，切换/被唤醒", self.core_id.0, self.threads[pos].id);

        let old_pos = self.current;
        self.threads[old_pos].state = State::Ready;

        self.threads[pos].state = State::Running;
        self.current = pos;
        unsafe {
            let old: *mut ThreadContext = self.threads[old_pos].ctx.as_mut().get_mut();
            let new: *const ThreadContext = self.threads[pos].ctx.as_ref().get_ref();
            llvm_asm!(
                "mov $0, %rdi
                 mov $1, %rsi"::"r"(old), "r"(new)
            );
            switch();
        }
    }

    /// 阻塞当前线程
    pub fn block(&mut self) {
        let pos = 0;

        let old_pos = self.current;
        self.threads[old_pos].state = State::Ready;

        //let sp_val = self.threads[old_pos].ctx.rsp;
        //let mask = (1 << 28) - 1;
        //let tid = (sp_val & mask) >> 20;
        println!("core: {}, 线程: {}，被阻塞", self.core_id.0, self.threads[old_pos].id);

        self.threads[pos].state = State::Running;
        self.current = pos;

        let mut t = self.threads.drain(old_pos..=old_pos).collect::<Vec<MyThread>>();
        //self.threads.push(t);
        //let len = self.threads.len();
        self.blocked.append(&mut t);
        let len = self.blocked.len();

        unsafe {
            let old: *mut ThreadContext = self.blocked[len-1].ctx.as_mut().get_mut();
            let new: *const ThreadContext = self.threads[pos].ctx.as_ref().get_ref();
            llvm_asm!(
                "mov $0, %rdi
                 mov $1, %rsi"::"r"(old), "r"(new)
            );
            switch();
        }
    }

    /// 唤醒阻塞队列中的线程
    pub fn notify(&mut self) {
        self.threads.append(&mut self.blocked);
    }

    /// 切换线程执行
    #[allow(dead_code)]
    pub fn thread_run(&mut self) {
        let mut pos = 1;
        while pos < self.threads.len() {
            if self.threads[pos].state == State::Ready { break; }
            pos += 1;
        }

        if pos == self.threads.len() {
            // 如果没有可以执行的线程，退出
            return ;
        }
        
        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old: *mut ThreadContext = self.threads[old_pos].ctx.as_mut().get_mut();
            let new: *const ThreadContext = self.threads[pos].ctx.as_ref().get_ref();
            llvm_asm!(
                "mov $0, %rdi
                 mov $1, %rsi"::"r"(old), "r"(new)
            );
            switch();
        }  
    }

    /// 由于运行时线程列表为空，且任务队列不空，创建一个线程插入到列表准备执行
    pub fn spawn(&mut self, f:fn() )
    {
        let mut available = MyThread::new(self.threads.len() + self.blocked.len());

        let size = available.stack.len();
        unsafe {
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }
        available.state = State::Ready;

        // 插入运行时
        self.threads.push(available);
    }
}

#[naked]
#[allow(unsupported_naked_functions)]
fn skip() { }

/// 任务队列为空，thread_main退出循环并结束，返回运行时寻找可以执行的下一个线程
fn guard() {
    unsafe {
        let rt_ptr = RUNTIME.with(|r| *r.borrow()) as *mut Runtime;
        (*rt_ptr).t_return();
    };
    
}

#[allow(dead_code)]
pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME.with(|r| *r.borrow()) as *mut Runtime;
        (*rt_ptr).t_yield();
    };
}
