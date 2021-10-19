#![feature(llvm_asm, naked_functions)]

mod task;
mod my_thread;
mod scheduler;
mod runtime;

use std::thread;

use runtime::{RUNTIME, Runtime};
use task::{Task};

use crate::{scheduler::{Scheduler}};

pub static mut NOYIFY: usize = 0;

pub fn block() {
    unsafe {
        let rt_ptr = RUNTIME.with(|r| *r.borrow()) as *mut Runtime;
        (*rt_ptr).block();
    };
}

pub fn notify() {
    unsafe { NOYIFY = 1; }
}

fn main() {
    // 启动调度器
    let sched = Scheduler::new();
    sched.init();

    // 创建协程
    Task::new(|| {
        println!("  协程 1 开始");

        println!("  协程 1 被阻塞");
        block();

        println!("  协程 1 被唤醒");
        println!("  协程 1 结束");

    }, 0);

    // 创建一个协程计算fbnc数列，每进行一次加法会主动让出
    Task::new_fnbc(5,0);

    Task::new(|| {
        println!("  协程 2 开始");

        println!("创建协程 2-1");
        Task::new(|| {
            println!("  协程 2-1 开始");

            println!("  协程 1 被唤醒");
            notify();

            println!("  协程 2-1 结束");
        }, 0);

        println!("  协程 2 结束");
    }, 0);
  
    for i in 3..20 {
        Task::new(move || {
            println!("协程 {} ", i);
        }, 0);
    }
    
    let mut ts = vec![];
    for i in 0..4 {
        let t = thread::spawn(move ||{
            let mut rt = Runtime::new();
            rt.init();
            rt.run();
            println!("CPU {} 结束", i + 1);
        });
        ts.push(t);
    }

    ts.into_iter().for_each(|h| h.join().unwrap());
}

