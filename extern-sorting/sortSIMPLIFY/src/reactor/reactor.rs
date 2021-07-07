use std::{collections::HashMap, fs, io::{Read, Seek, SeekFrom}, mem, sync::{Arc, Mutex, mpsc::{Sender, channel}}, task::Waker, thread::{self, JoinHandle}, time::Instant};

// 读buffer大小
const BUFFER_SIZE:usize = 1048576;

#[derive(Debug)]
pub enum Event {
    Close,
    // task_id,指定文件，指定位置，缓冲区
    ReadBuf(usize,String,u64,Arc<String>),
    ReadFile(usize,String,Arc<Mutex<String>>),
}

pub enum TaskState {
    Ready,
    NotReady(Waker),
    Finished,
}

pub struct Reactor {
    pub dispatcher: Sender<Event>,
    pub handle: Option<JoinHandle<()>>,
    pub tasks: HashMap<usize, TaskState>,
}

#[allow(unused_must_use)]
impl Reactor {
    pub fn new() -> Arc<Mutex<Box<Self>>> {
        // 初始化 dispatcher 和 tasks 
        let (tx, rx) = channel::<Event>();
        let reactor = Arc::new(Mutex::new(Box::new(Reactor {
            dispatcher: tx,
            handle: None,
            tasks: HashMap::new(),
        })));

        let reactor_clone = Arc::downgrade(&reactor);

        // 创建reactor线程
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut handles = vec![];

            for event in rx {
                let reactor = reactor_clone.clone();
                match event {
                    Event::Close => break,
                    Event::ReadBuf(id,file_name,position,result) => {
                        // Reactor线程接收到读文件事件
                        let event_handle = thread::spawn(move || {
                            println!("{} read start  {}",id,start.elapsed().as_secs_f64());
                            read_event(id, file_name, position, result,reactor.upgrade().unwrap());
                            println!("{} read finish {}",id,start.elapsed().as_secs_f64());
                        });
                        handles.push(event_handle);
                    }
                    Event::ReadFile(id,file_name,result) => {
                        let event_handle = thread::spawn(move || {
                            // 读文件
                            result.lock().map(|mut r| {
                                *r = fs::read_to_string(file_name).unwrap();
                            }).unwrap();
                            
                            // 调用Reactor::wake函数唤醒Task
                            let reactor = reactor.upgrade().unwrap();
                            reactor.lock().map(|mut r| r.wake(id)).unwrap();
                        });
                        handles.push(event_handle);
                    }
                }
            }

            handles.into_iter().for_each(|handle| handle.join().unwrap());
            println!("exit reactor_handle");
        });

        reactor.lock().map(|mut r| r.handle = Some(handle)).unwrap();
        // 返回
        reactor
    }

    /* 
     * 唤醒id对应的task：
     *   1.取出该task的state值。
     *   2.修改state值为Ready。
     *   3.如果旧值为NotReady，调用Waker::wake函数唤醒task；如果旧值为Finished，panic。
     *
     * 函数调用流程：wake() --> Waker::wake() -函数指针-> mywaker_wake() -最终调用-> unpark()  
     */
     pub fn wake(&mut self, id: usize) {
        let state = self.tasks.get_mut(&id).unwrap();
        match mem::replace(state, TaskState::Ready) {
            TaskState::NotReady(waker) => waker.wake(),
            TaskState::Finished => panic!("Called 'wake' twice on task: {}", id),
            _ => unreachable!()
        }
    }

    /* 
     * 向Reactor注册新的task：
     *   1. 向Reactor中插入新的<task_id,NotReady状态>s
     *   2. 发送事件给reactor线程
     */
     #[allow(unused_must_use)]
    pub fn register_buffer(&mut self,id: usize,waker: Waker,file_name:String,position:u64,buffer: Arc<String>){
        if self.tasks.insert(id, TaskState::NotReady(waker)).is_some() {
            panic!("Tried to insert a task with id: '{}', twice!", id);
        }
        self.dispatcher.send(Event::ReadBuf(id,file_name,position,buffer));
    }

    pub fn register_file(&mut self,id: usize,waker: Waker,file_name:String,buffer:Arc<Mutex<String>>){
        if self.tasks.insert(id, TaskState::NotReady(waker)).is_some() {
            panic!("Tried to insert a task with id: '{}', twice!", id);
        }
        self.dispatcher.send(Event::ReadFile(id,file_name,buffer));
    }

    // 检查id对应的task的state是否为ready，是则返回true，否则返回false。
    /* 变量类型Option<T>，取值有Some<T>、None，表示：有一个T类型的值、没有这样的值
     * None.unwrap()会触发panic()
     * 
     * Option变量调用map，变量直接作为参数传入闭包
     * 
     */
    pub fn is_ready(&self, id: usize) -> bool {
        self.tasks.get(&id).map(|state| match state {
            TaskState::Ready => true,
            _ => false,
        }).unwrap_or(false)
    }
}

#[allow(unused_must_use)]
pub fn read_event(id:usize,file_name:String,position:u64,result:Arc<String>,reactor: Arc<Mutex<Box<Reactor>>>){
    let start = Instant::now();
    let t0 = start.elapsed().as_secs_f64();

    let mut result_weak = Arc::downgrade(&result);
   
    // 打开文件
    let mut file = fs::File::open(file_name.clone()).unwrap();
    file.seek(SeekFrom::Start(position));
    // 读文件
    static mut BUFFER: [u8; BUFFER_SIZE] = [32u8;BUFFER_SIZE]; 
    unsafe {
        let len = file.read(&mut BUFFER).unwrap();
        match len {
            0 => result_weak = Arc::downgrade(&Arc::new("".to_string())),
            _ => result_weak = Arc::downgrade(&Arc::new(String::from_utf8(BUFFER.to_vec()).unwrap())),
        }
    }
    
   println!("read {} consume  {}",id,start.elapsed().as_secs_f64() - t0);
    // 调用Reactor::wake函数唤醒Task
    reactor.lock().map(|mut r| r.wake(id)).unwrap();
}
