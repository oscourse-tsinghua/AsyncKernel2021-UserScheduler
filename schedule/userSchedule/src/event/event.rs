#[derive(Clone)]
pub enum Event {
    // 关闭Reactor
    Close,
    // taskid,线程、协程的ID、当前时间、自己的字符串（行号和函数名）
    Print(usize,usize)
}

#[allow(non_snake_case)]
pub fn printEvent(tid:usize, cid:usize){
    println!("线程 {}, 协程 {}",tid,cid);
}