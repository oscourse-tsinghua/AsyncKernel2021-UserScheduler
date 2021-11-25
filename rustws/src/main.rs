#![feature(llvm_asm, naked_functions)]

mod task;
mod my_thread;
mod scheduler;
mod runtime;

use std::{net::{TcpListener}, thread};
use runtime::{RUNTIME, Runtime};
use smol::{Async, io};
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

#[allow(unused_must_use)]
fn main() {
    // 启动调度器
    let sched = Scheduler::new();
    sched.init();

    let mut handles = vec![];
    // 创建线程接收request
    for _ in 0..1 {
        let t = thread::spawn( || {
            handle_connect();
        });
        handles.push(t);
    }

    let mut ts = vec![];
    // 创建线程处理request
    for _ in 0..1 {
        let t = thread::spawn(move ||{
            let mut rt = Runtime::new();
            rt.init();
            rt.run();
            //println!("core: {} 结束", i + 1);
        });
        ts.push(t);
    }

    handles.into_iter().for_each(|h| h.join().unwrap());
    ts.into_iter().for_each(|h| h.join().unwrap());
    
}

fn handle_connect() -> io::Result<()> {
    let mut count: i64 = 0;

    smol::block_on(async {
        let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 9090) )?;

        loop {
            count = count + 1;
            let count_n = Box::new(count);
            let (stream, _) = listener.accept().await?;
            // 创建线程执行future
            Task::new_connect(stream, count_n, 0);
        }
    })
}

