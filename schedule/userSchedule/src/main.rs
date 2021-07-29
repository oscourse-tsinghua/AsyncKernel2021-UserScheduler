#![feature(llvm_asm, naked_functions)]
pub mod runtime{
    pub mod runtime;
}
pub mod reactor{
    pub mod reactor;
}
pub mod event{
    pub mod event;
}
pub mod task{
    pub mod task;
}
pub mod excutor{
    pub mod excutor;
}
pub mod join{
    pub mod join;
}
pub mod coroutine{
    pub mod coroutine;
}

use std::{collections::linked_list, future::{Future}, sync::{Arc, Mutex}};

use event::event::Event;
use reactor::reactor::Reactor;
use task::task::{Task,PriorityMap};
use runtime::runtime::{*};
use excutor::excutor::my_run_util;
//use join::join::join_all;

use crate::{coroutine::coroutine::{CCB, getPriorityByCid}, join::join::join_all, task::task::{Task_future, t1_map_ptr, t2_map_ptr}};

static mut rptr1:usize = 0;
static mut rptr2:usize = 0;

fn main() {
    
    // 创建reactor
    let reactor = Reactor::new();
    let reactor1 = Arc::clone(&reactor);
    let reactor2 = Arc::clone(&reactor);
    let x1:*const Arc<Mutex<Box<Reactor>>> = &reactor1;
    let x2:*const Arc<Mutex<Box<Reactor>>> = &reactor2;
    unsafe {
        rptr1 = x1 as usize;
        rptr2 = x2 as usize;
    }
    // 创建优先级map
    let mut map1 = PriorityMap::new();
    let mut map2 = PriorityMap::new();
    let map1_ptr:*const PriorityMap = &map1;
    let map2_ptr:*const PriorityMap = &map2;
    unsafe {
        t1_map_ptr = map1_ptr as usize;
        t2_map_ptr = map2_ptr as usize;
    }

    // 创建runtime
    let mut runtime = Runtime::new();
    // 初始化全局指针RUNTIME
    runtime.init();
    // 创建任务
    runtime.spawn(|| {
        println!("线程1 开始");

        // 创建任务
        let futs = creatTask1(4, 1);
        // 插入fut调度
        let fut_scheduler = async {
            join_all(futs,1).await;
        };
        println!("线程1 yield");
        yield_thread();
        // 把block_on创建为用户线程
        my_run_util(fut_scheduler);

        println!("线程1 结束");
        
    }); 

    runtime.spawn(|| {
        println!("线程2 开始");
        // 创建任务
        let futs = creatTask2(4, 2);
        // 插入fut调度
        let fut_scheduler = async {
            join_all(futs,2).await;
        };
        println!("线程2 yield");
        yield_thread();
        // 把block_on创建为用户线程
        my_run_util(fut_scheduler);
        println!("线程2 结束");
    }); 

    runtime.run();

}

fn creatTask1(n:usize, tid:usize) -> Vec<impl Future<Output = ()>>{
    let mut tasks = vec![];
    /* t1
     * tid = 1
     * cid = 1..4
     * task_id = 0..3
     * priority = 4..1
     */
    for i in 0..n {
        // 克隆tid,cid
        let cid = (i+1).clone();
        let t = tid.clone();
        // 创建协程
        let task = async move {
            println!("线程{}   协程{}   优先级{}",t,cid,getPriorityByCid(t, cid));
        };
        tasks.push(task);
        
    }
    // 创建CCB，并插入全局链表
    set_priority1();
    tasks
}

fn set_priority1(){
    CCB::new(1, 1, 2, 0).insert();
    CCB::new(1, 2, 4, 1).insert();
    CCB::new(1, 3, 1, 2).insert();
    CCB::new(1, 4, 5, 3).insert();
}

fn creatTask2(n:usize, tid:usize) -> Vec<impl Future<Output = ()>>{
    let mut tasks = vec![];
    /* t2
     * tid = 2
     * cid = 1..4
     * task_id = 4..7
     * priority = 4..1
     */
    for i in 0..n {
        // 克隆tid,cid
        let cid = (i+1).clone();
        let t = tid.clone();
        // 创建协程
        let task = async move {
            println!("线程{}   协程{}   优先级{}",t,cid,getPriorityByCid(t, cid));
        };
        tasks.push(task);
        
    }
    // CCB
    set_priority2();
    tasks
}

fn set_priority2(){
    CCB::new(2, 1, 5, 0).insert();
    CCB::new(2, 2, 3, 1).insert();
    CCB::new(2, 3, 2, 2).insert();
    CCB::new(2, 4, 4, 3).insert();
}





