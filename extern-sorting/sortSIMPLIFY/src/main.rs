pub mod reactor{
    pub mod reactor;
}

pub mod excutor{
    pub mod excutor;
} 
use excutor::excutor::*;
use futures::{Future};
use reactor::reactor::*;

use std::{fs,pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll}, thread, time::Instant};

#[allow(unused_must_use)]
fn main() {
    // 开启计时
    let start = Instant::now();
    // 创建reactor
    let reactor = Reactor::new();
    let reactor1 = Arc::clone(&reactor);
    let reactor2 = Arc::clone(&reactor);
    // 创建id
    let id = Arc::new(Mutex::new(1));
    let id1 = Arc::clone(&id);
    let id2 = Arc::clone(&id);

    let t1 = thread::spawn(move || {
        // fut1,fut2创建线程读sort1，分为多个buffer读入后，排序写入临时文件
        let id_arg = Arc::clone(&id1);
        let reactor_arg = Arc::clone(&reactor1);
        // 创建多协程读
        let fut_thread = async move{
            println!("t1 start  {}",start.elapsed().as_secs_f64());
            let file_name = "sort1.txt".to_string();
            // 生成多个读协程
            let futs = readfut_spawn(file_name.clone(), id_arg.clone(), reactor_arg.clone());
            // 并发执行读协程
            futures::future::join_all(futs).await;
            println!("t1 finish {}",start.elapsed().as_secs_f64());
        };
        my_run_util(fut_thread);
        
    });

    let t2 = thread::spawn(move || {
        // fut1,fut2创建线程读sort1，分为多个buffer读入后，排序写入临时文件
        let id_arg = Arc::clone(&id2);
        let reactor_arg = Arc::clone(&reactor2);
        // 创建多协程读
        let fut_thread = async move{
            println!("t2 start  {}",start.elapsed().as_secs_f64());
            let file_name = "sort2.txt".to_string();
            // 生成多个读协程
            let futs = readfut_spawn(file_name.clone(), id_arg.clone(),  reactor_arg.clone());
            // 并发执行读协程
            futures::future::join_all(futs).await;
            println!("t2 finish {}",start.elapsed().as_secs_f64());
        };
        my_run_util(fut_thread);
        
    });

    t1.join();
    t2.join();
}

// 游标步长、读buffer大小
// 40M : 41943040
// 16M : 16777216
// 8M  : 8388608 
// 4M  : 4194304
// 2M  : 2097152
// 1M  : 1048576
const BUFFER_SIZE:usize = 1048576;

#[allow(unused_must_use)]
#[allow(unused_variables)]
// 生成读文件协程
fn readfut_spawn(file_name:String,id1:Arc<Mutex<usize>>,reactor1: Arc<Mutex<Box<Reactor>>>) -> Vec<impl Future>{
    let mut fut_vec = vec![];
    // 获取文件大小
    let file = fs::File::open(file_name.clone()).unwrap();
    let file_size = file.metadata().unwrap().len();
    // 计算读线程个数
    let n = file_size / BUFFER_SIZE as u64;
    println!("file {} n = {}",file_name,n);
    // 读位置
    let mut position:u64 = 0;

    for i in 0..n {
        // 打开相同文件，写入不同临时文件
        let count = (i+1).to_string().clone();
        let reactor = Arc::clone(&reactor1);
        let id = id1.lock().map(|mut id| {*id += 1;*id-1}).unwrap();
        let file_name = file_name.clone();
        let start = Instant::now();
        // 设置读起始位置
        let fut_pos = position;
        position += BUFFER_SIZE as u64;

        let fut = async move {
            let count_fut = count;
            // 指定缓冲区
            let mut buffer = Arc::new("".to_string());
            // 发送读文件事件
            read_buf(file_name.clone(), fut_pos , buffer.clone(), id, reactor).await;
            // 取出读线程读到的数据
            let read_result = (*buffer).clone();

        };

        fut_vec.push(fut);
    }
    fut_vec
}

// 发送读文件事件
#[allow(unused_variables)]
async fn read_buf(file_name:String,position:u64,buffer: Arc<String>,id: usize,reactor: Arc<Mutex<Box<Reactor>>>){
    let task = Task_Buffer::new(id, file_name, position, buffer,reactor).await;
}

#[allow(non_camel_case_types)]
pub struct Task_Buffer {
    id: usize,
    file_name: String,
    position: u64,
    buffer: Arc<String>,
    reactor: Arc<Mutex<Box<Reactor>>>,
}

impl Task_Buffer {
    fn new(id:usize, file_name:String, position:u64, buffer:Arc<String>, reactor:Arc<Mutex<Box<Reactor>>>) -> Self{
        Task_Buffer{id, file_name, position, buffer,reactor}
    }
}

impl Future for Task_Buffer {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut r = self.reactor.lock().unwrap();
        if r.is_ready(self.id) {
            //*r.tasks.get_mut(&self.id).unwrap() = TaskState::Finished;
            r.tasks.remove(&self.id);
            // 将退回到Fut1中，把self.id返回给val
            Poll::Ready(self.id)
        } else if r.tasks.contains_key(&self.id) {
            r.tasks.insert(self.id, TaskState::NotReady(cx.waker().clone()));
            Poll::Pending
        } else {
            r.register_buffer(self.id,cx.waker().clone(),self.file_name.clone(),self.position,self.buffer.clone());
            Poll::Pending
        }
    }
}

