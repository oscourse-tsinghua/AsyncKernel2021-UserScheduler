pub mod fileop{
    pub mod fileop;
}

pub mod reactor{
    pub mod reactor;
}

pub mod excutor{
    pub mod excutor;
} // 导包

use std::{ fs, ops::Sub, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll}, thread::{self}, time::Instant};

use excutor::excutor::*;
use fileop::fileop::*;
use futures::{Future};
use reactor::reactor::*;


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

    let thread1 = thread::spawn(move || {
        // fut1,fut2创建线程读sort1，分为多个buffer读入后，排序写入临时文件
        let position:Arc<Mutex<u64>> = Arc::new(Mutex::new(0));  
        let read_len:Arc<Mutex<u64>> = Arc::new(Mutex::new(1));
        let id_arg = Arc::clone(&id1);
        let reactor_arg = Arc::clone(&reactor1);
        /* 
        // merge: 每个read读入一个临时文件
        let fut_merge = async {
            let buffer = Arc::new(Mutex::new("".to_string()));
            let mut read_vec = vec![];
            for i in 0..2 {
                // 打开不同文件
                let count = (i+1).to_string().clone();
                let file_name = "data1_".to_string() + &(i.clone()+1).to_string() + &".txt".to_string();
                let read = async {
                    let count_read = count;
                    println!("t1 merge read{} start {}",count_read,start.elapsed().as_secs_f64());
                    // 初始化读file参数
                    let id = id1.lock().map(|mut id| {*id += 1;*id-1}).unwrap();
                    let buffer_temp = Arc::new(Mutex::new("".to_string()));
                    // 读file
                    read_file(file_name, 0, buffer_temp.clone(), id, reactor1.clone()).await;
                    // 合并结果
                    let result = buffer_temp.lock().map(|b| b.to_string()).unwrap();
                    buffer.lock().map(|mut b|b.push_str(&result));
                    println!("t1 merge read{} finish {}",count_read,start.elapsed().as_secs_f64());
                };

                read_vec.push(read);
            }
            
            println!("t1 merge join2 start         {}",start.elapsed().as_secs_f64());
            futures::future::join_all(read_vec).await;
            println!("t1 merge join2 finish        {}",start.elapsed().as_secs_f64());
            // string => vec
            let mut nums = string_to_vec(buffer.lock().map(|b| b.to_string()).unwrap());
            // 排序
            nums.sort();
            // 写nums
            let write_file = "result1.txt".to_string();
            write_vec(nums, write_file);
        };
        */

        let fut_thread = async move{
            println!("t1 fut join2 start          {}",start.elapsed().as_secs_f64());
            loop {
                let futs = readfut_spawn2(start, id_arg.clone(), position.clone(), reactor_arg.clone(), read_len.clone());
                futures::future::join_all(futs).await;
                let len = read_len.lock().map(|l| *l).unwrap();
                if len == 0 {break;}
            }
            
            println!("t1 fut read sort1 finish         {}",start.elapsed().as_secs_f64());
            //fut_merge.await;
        };

        my_run_util(fut_thread);

    });
    
     
    let thread2 = thread::spawn(move || {
        let position = Arc::new(Mutex::new(0));
        let read_len:Arc<Mutex<u64>> = Arc::new(Mutex::new(1));
        let id_arg = Arc::clone(&id2);
        let reactor_arg = Arc::clone(&reactor2);
        /* 
        let fut_merge = async {
            let buffer = Arc::new(Mutex::new("".to_string()));
            let mut read_vec = vec![];

            for i in 0..5 {
                let count = (i+1).to_string().clone();
                let file_name = "data2_".to_string() + &(i.clone()+1).to_string() + &".txt".to_string(); 
                let read = async {
                    let count_read = count;
                    println!("t2 merge read{} start {}",count_read,start.elapsed().as_secs_f64());
                    // 初始化读file参数
                    let id = id2.lock().map(|mut id| {*id += 1;*id-1}).unwrap();
                    let buffer_temp = Arc::new(Mutex::new("".to_string()));
                    // 读file
                    read_file(file_name, 0, buffer_temp.clone(), id, reactor2.clone()).await;
                    println!("t2 merge read{} finish {}",count_read,start.elapsed().as_secs_f64());
                    // 合并结果
                    let result = buffer_temp.lock().map(|b| b.to_string()).unwrap();
                    buffer.lock().map(|mut b|b.push_str(&result));
                
                    /* 直接读文件 
                    let result_no_reactor = read_file_no_reactor(file_name);
                    println!("t2 merge read{} finish {}",count_read,start.elapsed().as_secs_f64());
                    buffer.lock().map(|mut b|b.push_str(&result_no_reactor));
                    */
                };

                read_vec.push(read);
            }
            
            println!("t2 merge join5 start         {}",start.elapsed().as_secs_f64());
            futures::future::join_all(read_vec).await;
            println!("t2 merge join5 finish        {}",start.elapsed().as_secs_f64());
            // string => vec
            let mut nums = string_to_vec(buffer.lock().map(|b| b.to_string()).unwrap());
            // 排序
            nums.sort();
            // 写nums
            let write_file = "result2.txt".to_string();
            write_vec(nums, write_file);
        };
        */

        let fut_thread = async {
            println!("t2 fut join5 start          {}",start.elapsed().as_secs_f64());
            loop {
                let futs = readfut_spawn5(start, id_arg.clone(), position.clone(), reactor_arg.clone(), read_len.clone());
                futures::future::join_all(futs).await;
                //futures::future::join_all(futs).await;
                let len = read_len.lock().map(|l| *l).unwrap();
                if len == 0 {break;}
            }
            println!("t2 fut read sort2 finish         {}",start.elapsed().as_secs_f64());
            //fut_merge.await;
        };

        my_run_util(fut_thread);

    });
    
    thread1.join();
    thread2.join();
    
}

// 游标步长、读buffer大小
const BUFFER_SIZE:usize = 1048576;

// 生成读文件协程
#[warn(unused_must_use)]
fn readfut_spawn2(start1:Instant,id1:Arc<Mutex<usize>>,position1:Arc<Mutex<u64>>,reactor1: Arc<Mutex<Box<Reactor>>>,read_len:Arc<Mutex<u64>>) -> Vec<impl Future>{
    let mut fut_vec = vec![];
    
    for i in 0..3 {
        // 打开相同文件，写入不同临时文件
        let count = (i+1).to_string().clone();
        let reactor = Arc::clone(&reactor1);
        let position = Arc::clone(&position1);
        let start = start1.clone();
        let id = id1.lock().map(|mut id| {*id += 1;*id-1}).unwrap();
        let rd_len = Arc::clone(&read_len);

        let fut = async move {
            let count_fut = count;
            //println!("t1 fut{} read start {}",count_fut,start.elapsed().as_secs_f64());
            // 初始化读函数参数: 指定文件，指定位置，指定缓冲区
            let file_name = "sort1.txt".to_string();
            let pos = position.lock().map(|mut p| {*p += BUFFER_SIZE as u64;p.sub(BUFFER_SIZE as u64)}).unwrap();
            let buffer = Arc::new(Mutex::new("".to_string()));
            // 读文件
            read_buf(file_name.clone(), pos, buffer.clone(), id, reactor).await;
            // 如果读出长度为 0 ，读文件完毕，退出循环
            let result_buf = buffer.lock().map(|b| (*b).clone()).unwrap();
            
            let len = result_buf.len() as u64;
            // 当其他读函数读出长度为0时，就不再更新read_len
            rd_len.lock().map(|mut r| {if *r != 0 {*r = len}});
            //println!("t1 fut{} read finish {}",count_fut,start.elapsed().as_secs_f64());
            // string => vec
            //let mut nums = string_to_vec(result);
            // 排序
            //nums.sort();
            // 写入临时文件
            //let write_file = "data1_".to_string() + &count_fut + &".txt".to_string();
            //write_vec(nums, write_file);

            //println!("t1 fut{} over {}",count_fut,start.elapsed().as_secs_f64());
        };

        fut_vec.push(fut);
    }
    fut_vec
}

/* 
fn tttttest(file_name:String,position:u64,buffer: Arc<Mutex<String>>,id: usize,reactor: Arc<Mutex<Box<Reactor>>>) {
    let handle = thread::spawn(move || {
        // 打开文件
        let mut file = fs::File::open(file_name).unwrap();
        // 创建buffer,大小为4k
        let mut buf = [32u8;409600];
        // 设置游标
        file.seek(SeekFrom::Start(position));
        // 读文件
        let len = file.read(&mut buf).unwrap();
        // 传回结果
        match len {
            0 => buffer.lock().map(|mut r| r.push_str(&"".to_string())).unwrap(),
            _ => buffer.lock().map(|mut r| r.push_str(&String::from_utf8(buf.to_vec()).unwrap() ) ).unwrap(),
        }
        
    });

    yield_now();

    handle.join();

}
*/

#[warn(unused_must_use)]
fn readfut_spawn5(start1:Instant,id1:Arc<Mutex<usize>>,position1:Arc<Mutex<u64>>,reactor1: Arc<Mutex<Box<Reactor>>>,read_len:Arc<Mutex<u64>>) -> Vec<impl Future>{
    let mut fut_vec = vec![];
    
    for i in 0..9 {
        // 打开相同文件，写入不同临时文件
        let count = (i+1).to_string().clone();
        let reactor = Arc::clone(&reactor1);
        let position = Arc::clone(&position1);
        let start = start1.clone();
        let id = id1.lock().map(|mut id| {*id += 1;*id-1}).unwrap();
        let rd_len = Arc::clone(&read_len);

        let fut = async move {
            let count_fut = count;
            //println!("t2 fut{} read start {}",count_fut,start.elapsed().as_secs_f64());
            // 初始化读函数参数: 指定文件，指定位置，指定缓冲区
            let file_name = "sort2.txt".to_string();
            let pos = position.lock().map(|mut p| {*p += BUFFER_SIZE as u64;p.sub(BUFFER_SIZE as u64)}).unwrap();
            let buffer = Arc::new(Mutex::new("".to_string()));
            // 读文件
            read_buf(file_name.clone(), pos, buffer.clone(), id, reactor).await;
            // 如果读出长度为 0 ，读文件完毕，退出循环
            let result_buf = buffer.lock().map(|b| (*b).clone()).unwrap();
            let mut len = result_buf.len() as u64;
            
            // 当其他读函数读出长度为0时，就不再更新read_len
            rd_len.lock().map(|mut r| {if *r != 0 {*r = len}});
            //println!("t2 fut{} read finish {}",count_fut,start.elapsed().as_secs_f64());
            // string => vec
            //let mut nums = string_to_vec(result);
            // 排序
            //nums.sort();
            // append模式写入临时文件
            //let write_file = "data2_".to_string() + &count_fut + &".txt".to_string();
            //write_vec(nums, write_file);

            //println!("t2 fut{} over {}",count_fut,start.elapsed().as_secs_f64());
        };

        fut_vec.push(fut);
    }
    fut_vec
}


// 读buffer
#[allow(unused_variables)]
async fn read_buf(file_name:String,position:u64,buffer: Arc<Mutex<String>>,id: usize,reactor: Arc<Mutex<Box<Reactor>>>){
    let task = Task_Buffer::new(id, file_name, position, buffer,reactor).await;
}
// 读file
#[allow(unused_variables)]
async fn read_file(file_name:String,buffer: Arc<Mutex<String>>,id: usize,reactor: Arc<Mutex<Box<Reactor>>>){
    let task = Task_File::new(id, file_name, buffer, reactor).await;
}

// 直接读文件
#[allow(dead_code)]
fn read_file_no_reactor(file_name:String) -> String {
    let buf = fs::read_to_string(file_name).unwrap();
    buf
}

pub struct Task_File {
    id: usize,
    file_name: String,
    buffer: Arc<Mutex<String>>,
    reactor: Arc<Mutex<Box<Reactor>>>,
}

impl Task_File {
    fn new(id:usize, file_name:String, buffer: Arc<Mutex<String>>, reactor:Arc<Mutex<Box<Reactor>>>) -> Self{
        Task_File{id, file_name, buffer,reactor}
    }
}

impl Future for Task_File {
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
            r.register_file(self.id,cx.waker().clone(),self.file_name.clone(),self.buffer.clone());
            Poll::Pending
        }
    }
}



pub struct Task_Buffer {
    id: usize,
    file_name: String,
    position: u64,
    buffer: Arc<Mutex<String>>,
    reactor: Arc<Mutex<Box<Reactor>>>,
}

impl Task_Buffer {
    fn new(id:usize, file_name:String, position:u64, buffer:Arc<Mutex<String>>, reactor:Arc<Mutex<Box<Reactor>>>) -> Self{
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

