use std::{collections::{HashMap}, mem, sync::{Arc, Weak,Mutex, mpsc::{Sender, channel}}, task::Waker, thread::{self, JoinHandle}};

use crate::{event::event::*, runtime::runtime::{RUNTIME, Runtime}};

#[derive(Debug)]

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

static mut rw_ptr:usize = 0;

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
            //let start = Instant::now();
            for event in rx {
                let reactor = reactor_clone.clone();
                // 设置全局指针
                let rw_ptr_usize:*const Weak<Mutex<Box<Reactor>>> = &reactor;
                unsafe {
                    rw_ptr = rw_ptr_usize as usize;
                }
                // 处理事件
                match event {
                    Event::Close => break,
                    Event::Print(Tid, Cid) => {
                        unsafe {
                            let rt_prt = RUNTIME as *mut Runtime;
                            match Tid {
                                1 => {
                                    match Cid {
                                        1 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(1, 1);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(0)).unwrap();
                                            });
                                        }
                                        2 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(1, 2);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(1)).unwrap();
                                            });
                                        }
                                        3 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(1, 3);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(2)).unwrap();
                                            });
                                        }
                                        4 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(1, 4);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(3)).unwrap();
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                                2 => {
                                    match Cid {
                                        1 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(2, 1);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(4)).unwrap();
                                            });
                                        }
                                        2 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(2, 2);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(5)).unwrap();
                                            });
                                        }
                                        3 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(2, 3);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(6)).unwrap();
                                            });
                                        }
                                        4 => {
                                            (*rt_prt).spawn(||{
                                                printEvent(2, 4);
                                                // 唤醒task
                                                let rw = rw_ptr as *mut Weak<Mutex<Box<Reactor>>>;
                                                let rct = (*rw).upgrade().unwrap();
                                                rct.lock().map(|mut r| r.wake(7)).unwrap();
                                            }); 
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {

                                }
                            }
                        }
                    }  
                }
            }

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
    pub fn register(&mut self, id: usize, waker: Waker, event: Event){
        if self.tasks.insert(id, TaskState::NotReady(waker)).is_some() {
            panic!("Tried to insert a task with id: '{}', twice!", id);
        }
        self.dispatcher.send(event);
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
