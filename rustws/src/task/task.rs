use std::{fs, future::Future, io::Write, net::TcpStream, pin::Pin, sync::{ Mutex, atomic::AtomicUsize}, time::Duration};

use smol::{Async, io::AsyncReadExt};

use crate::scheduler::{SCHED, Scheduler};

pub struct TaskId(pub usize);

#[allow(dead_code)]
impl TaskId {
    fn get_id() -> TaskId {
        // static变量只初始化一次，内核线程中，应使用 AtomicUsize 确保 COUNTER 线程安全
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        TaskId(id)
    }

    fn set_id(id: usize) -> TaskId {
        Self(id)
    }
}

/// 协程：保存在进程的地址空间中，进程与线程不需要有保存协程的列表，在对应的进程地址空间中，执行器可以直接执行协程
///
/// 优先级：0, 1, 2, 3, 默认情况下设置为 1。
pub struct Task {
    pub id: TaskId,
    pub future: Mutex<Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>>,
    pub process: usize,
    pub priority: usize,
}

#[allow(dead_code)]
impl Task {
    #[allow(unused_mut)]
    pub fn new_connect(mut stream: Async<TcpStream>, count: Box<i64>, pid: usize) {
        let num = *count;
        let task_future = async {
            handle_connection(stream, count).await;
        };

        let task = Task {
            id: TaskId::get_id(),
            future: Mutex::new(Box::pin(task_future)),
            process: pid,
            priority: 9,
        };
        
        // 取得调度器，插入任务队列
        unsafe {
            let s_ptr = SCHED as *mut Scheduler;
            (*s_ptr).push(task);
        }
    }

    
}

async fn handle_connection(mut stream: Async<TcpStream>, count: Box<i64>) {
    // Read the first 1024 bytes of data from the stream
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    // 每接收10个request，延迟2s，100个线程并发，近似于平均1000个request延迟2s
    if (*count % 10) == 0 {
        println!("Adding delay. Count: {}", count);
        smol::Timer::after(Duration::from_secs(2)).await;
    }

    let header = "
    HTTP/1.0 200 OK
    Connection: keep-alive
    Content-Length: 174
    Content-Type: text/html; charset=utf-8
        ";

    let file_name = "hi.html";
    let contents = fs::read_to_string(file_name).unwrap();

    let response = format!("{}\r\n\r\n{}", header, contents);

    stream.get_mut().write(response.as_bytes()).unwrap(); // write response
    stream.get_mut().flush().unwrap();
}



