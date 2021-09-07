use std::{pin::Pin, sync::{Arc, Mutex, atomic::AtomicUsize}};

use futures::{Future};

use super::{KernelThread};
use crate::runtime::{RUNTIME, Runtime};

pub struct TaskId(pub usize);

impl TaskId {
    fn get_id() -> TaskId {
        // static变量只初始化一次，内核线程中，应使用 AtomicUsize 确保 COUNTER 线程安全
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        TaskId(id)
    }
}

#[allow(dead_code)]
pub enum TaskState {
    Running,
    Ready,
}

// 协程
pub struct Task {
    pub id: TaskId,
    pub future: Mutex<Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>>,
    pub state: Mutex<TaskState>,
    pub thread: Arc<Mutex<KernelThread>>,
}


impl Task {
    /// 创建协程 + 插入调度队列 + 插入线程task列表
    /// Task存在于Thread的列表中，和任务队列中
    pub fn new(
        f: impl FnOnce() + 'static + Send + Sync
    ){

       let task_inner: impl Future<Output = ()> + 'static + Send + Sync = async {
           f();
       };

       // 取得调度器，拿到当前线程句柄
       let rt_ptr;
       let t;
       unsafe {
           rt_ptr = RUNTIME as *mut Runtime;
           t = (*rt_ptr).current.thread.clone();
       }

       let task = Arc::new(Task{
            id: TaskId::get_id(),
            future: Mutex::new(Box::pin(task_inner)),
            state: Mutex::new(TaskState::Ready),
            thread: t,
        });

        // 插入调度队列，插入线程task列表
        unsafe {
            (*rt_ptr).current.thread.lock().as_mut().unwrap().add_task(task.clone());
            (*rt_ptr).add_task(task);
        }
    }

    /// 创建协程，但不插入调度队列
    pub fn creat_task(
        f: impl FnOnce() + 'static + Send + Sync,
        _thread: Arc<Mutex<KernelThread>>
    ) -> Arc<Self> {

        let task_future: impl Future<Output = ()> + 'static + Send + Sync = async {
            f();
        };

        Arc::new(Task{
            id: TaskId::get_id(),
            future: Mutex::new(Box::pin(task_future)),
            state: Mutex::new(TaskState::Ready),
            thread: _thread,
        })
    }

    pub fn idle(
        f: impl FnOnce() + 'static + Send + Sync
    ) -> Arc<Self> {
        let task_inner: impl Future<Output = ()> + 'static + Send + Sync = async {
            f();
        };
 
        // 取得调度器，拿到当前线程句柄
        let t = KernelThread::idle_thread();
 
        Arc::new(Task{
             id: TaskId::get_id(),
             future: Mutex::new(Box::pin(task_inner)),
             state: Mutex::new(TaskState::Ready),
             thread: t,
        })
    }

    
}