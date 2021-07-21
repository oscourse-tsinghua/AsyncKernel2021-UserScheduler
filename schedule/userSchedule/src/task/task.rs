use std::{collections::HashMap, future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll}};

use crate::{event::event::Event, reactor::reactor::*, runtime::runtime::yield_thread};

pub struct Task{
    id:usize,
    event:Event,
    reactor:Arc<Mutex<Box<Reactor>>>,
}

impl Task{
    pub fn new(id:usize, event:Event, reactor:Arc<Mutex<Box<Reactor>>>) -> Self {
        Task{id,event,reactor}
    }
}

impl Future for Task {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut r = self.reactor.lock().unwrap();
        // 如果事件已经完成，删除此Task,并返回ready
        // 如果未完成，就注册
        if r.is_ready(self.id) {
            r.tasks.remove(&self.id);
            Poll::Ready(self.id)
        } else if r.tasks.contains_key(&self.id) {
            r.tasks.insert(self.id, TaskState::NotReady(cx.waker().clone()));
            Poll::Pending
        } else {
            r.register(self.id, cx.waker().clone(), self.event.clone());
            Poll::Pending
        }
    }
}

#[allow(non_camel_case_types)]
pub struct Task_future{
    id:usize,
    tid:usize,
    cid:usize,
    priority:usize,
}

impl Task_future{
    pub fn new(id:usize, tid:usize, cid:usize, priority:usize) -> Self {
        Task_future{id, tid, cid, priority}
    }
}

impl Future for Task_future {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        do_sth(self.tid, self.cid, self.priority);
        Poll::Ready(self.id)
    }
}

fn do_sth(t:usize, c:usize, p:usize){
    println!("线程{} 协程{} 优先级{} 开始",t,c,p);
    println!("线程{} 协程{} yield",t,c);
    yield_thread();
    println!("线程{} 协程{} 优先级{} 结束",t,c,p);
}

// 每个block_on一个map，通过vec下标映射到该future的优先级
#[derive(Clone)]
pub struct PriorityMap{
    pub pri:HashMap<usize, usize>,
    pub tier:usize,
}

impl PriorityMap {
    pub fn new() -> Self {
        PriorityMap {pri:HashMap::new(),tier:4}
    }
}

pub static mut t1_map_ptr:usize = 0;
pub static mut t2_map_ptr:usize = 0;