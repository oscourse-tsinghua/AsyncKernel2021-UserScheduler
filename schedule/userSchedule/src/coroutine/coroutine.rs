
use std::{pin::Pin, usize};

use futures::{Future, pin_mut};

use crate::{excutor::excutor::my_run_util, join::join::join_all};

/* 
pub struct Coroutine<'a>{
    tid:usize,
    futures_vec:Vec<Pin<&'a mut dyn Future<Output = ()>>>,
}

impl<'a> Coroutine<'a>{

    pub fn new(t:usize) -> Self {
        Coroutine{tid:t, futures_vec:vec![]}
    }

    // 创建协程，插入协程列表
    pub fn spawn(&mut self, p:usize, future:dyn Future<Output = ()>){

        pin_mut!(future);
        
        self.futures_vec.push();

        let len = self.futures_vec.len();
        // 创建CCB并插入CCB_list
        CCB::new(self.tid, len-1, p, len-1).insert();
    }

    pub fn run(self) {
        if self.futures_vec.len() != 0 {
            let future_main = async{
                join_all(self.futures_vec, self.tid).await;
            };
            my_run_util(future_main);
        }
    }

    fn get_pin_future(future:F) -> Pin<&'a mut dyn Future<Output = ()> {

    }
}
*/


#[derive(Clone, Copy)]
pub struct CCB{
    tid:usize,
    cid:usize,
    priority:usize,
    f_index:usize,
}

impl CCB{
    pub fn new(t:usize, c:usize, p:usize, f_i:usize) -> Self{
        CCB {tid:t, cid:c, priority:p, f_index:f_i}
    }

    pub fn insert(self){
        unsafe {
            let len = CCB_list.len();
            for i in 0..len+1 {
                // 将self按优先级从高到低插入到列表中
                if i == len || CCB_list[i].priority >= self.priority{
                    CCB_list.insert(i, self);
                    break ;
                }
            }
        }
    }
}

// CCB队列，按优先级从高到低排列
#[allow(non_upper_case_globals)]

pub static mut CCB_list:Vec<CCB> = vec![];
#[allow(non_snake_case)]
pub fn getPriorityByCid(t:usize, c:usize) -> usize{
    unsafe{
        for i in 0..CCB_list.len(){
            if CCB_list[i].tid == t && CCB_list[i].cid == c {
                return CCB_list[i].priority;
            }
        }
        0
    }
}

#[allow(non_snake_case)]
pub fn getPriorityByIndex(t:usize, index:usize) -> usize{
    unsafe{
        for i in 0..CCB_list.len(){
            if CCB_list[i].tid == t && CCB_list[i].f_index == index {
                return CCB_list[i].priority;
            }
        }
        0
    }
}

// 判断最高优先级协程是否位于另一线程内
#[allow(non_snake_case)]
pub fn checkYield(t:usize) -> bool {
    let mut maxTP = 999;
    let mut maxOherTP = 999;
    unsafe {
        // 切换: tid != t && 优先级更高 
       for i in 0..CCB_list.len() {
           if CCB_list[i].tid == t && CCB_list[i].priority < maxTP {
               maxTP = CCB_list[i].priority;
           }
           if CCB_list[i].tid != t && CCB_list[i].priority < maxOherTP{
               maxOherTP = CCB_list[i].priority;
           }
       }
    }
    // 优先级严格高时才切换
    if maxOherTP < maxTP{
        return true;
    }else{
        return false;
    }
}

#[allow(non_snake_case)]
pub fn removeCCB(t:usize, priority:usize){
    unsafe {
        for i in 0..CCB_list.len() {
            if CCB_list[i].tid == t && CCB_list[i].priority == priority {
                CCB_list.remove(i);
                //println!("删除 线程{} priority{}",t,priority);
                break;
            }
        }
    }
}