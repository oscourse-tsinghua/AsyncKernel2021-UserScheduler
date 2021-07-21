
use std::{future::Future, pin::Pin, sync::{Arc, Condvar, Mutex}, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};
use crate::runtime::runtime::yield_thread;

pub fn my_run_util<F: Future>(mut future: F) -> F::Output {
        // 从最内层parker开始创建上下文
        let parker = Arc::new(Parker::default());
        let mywaker = Arc::new(MyWaker { parker: parker.clone() });
        let waker = mywaker_into_waker(Arc::into_raw(mywaker));
        let mut cx = Context::from_waker(&waker);
    
        // SAFETY: we shadow `future` so it can't be accessed again.
        let mut future = unsafe { Pin::new_unchecked(&mut future) };
        loop {
            match Future::poll(future.as_mut(), &mut cx) {
                Poll::Ready(val) => break val,
                Poll::Pending => {
                    //parker.park();
                    yield_thread();
                    println!("block_on线程内所有协程都在等待,yield");
                }
            };
            
        }
    }

    #[derive(Default)]
    pub struct Parker(Mutex<bool>, Condvar);  //元组结构体，没有变量名
    
    /* Mutex<bool>：被用于wake()设置为true，标记为已唤醒
     * Condvar：
     *   - notify_one()：唤醒一个阻塞在此条件变量上的进程
     *   - wait()：阻塞当前线程直到notify
     * 
     * block_on中在loop中执行async的Future：
     *   - 返回Pending就调用park()，阻塞条件变量，并设置Mutex<bool>为false
     *   - 被wake()唤醒时，就调用unpark()，notify条件变量，并设置Mutex<bool>为true
     */
    impl Parker {
        fn park(&self) {
            let mut resumable = self.0.lock().unwrap(); 
            while !*resumable {
                    resumable = self.1.wait(resumable).unwrap();
                }
            *resumable = false;
        }
    
        fn unpark(&self) {
            *self.0.lock().unwrap() = true;
            self.1.notify_one();
        }
    }
   #[derive(Clone)]
   pub struct MyWaker {
       parker: Arc<Parker>, //Arc ：“Atomically Reference Counted/原子引用计数”。
   }

   // 调用unpark唤醒Task_Future与主线程
    pub fn mywaker_wake(s: &MyWaker) {
        let waker_arc = unsafe { Arc::from_raw(s) };
        waker_arc.parker.unpark();
    }

    pub fn mywaker_clone(s: &MyWaker) -> RawWaker {
        let arc = unsafe { Arc::from_raw(s) };
        std::mem::forget(arc.clone()); // increase ref count
        RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
    }

    /* 指向trait对象的指针布局：
 *   - 前8个B是数据。
 *   - 后8个B是vtable
 * 这四个函数指针是VTable接口的要求。
 */
    pub const VTABLE: RawWakerVTable = unsafe {
        RawWakerVTable::new(
            |s| mywaker_clone(&*(s as *const MyWaker)),   // clone
            |s| mywaker_wake(&*(s as *const MyWaker)),    // wake
            |s| (*(s as *const MyWaker)).parker.unpark(), // wake by ref (don't decrease refcount)
            |s| drop(Arc::from_raw(s as *const MyWaker)), // decrease refcount
        )
    };

    /* 使用std库的RawWaker封装自己的MyWaker，再把RawWaker封装进Waker
     * std库的Waker、RawWaker是标准库提供的接口，用来封装自己的MyWaker实现
     */
    pub fn mywaker_into_waker(s: *const MyWaker) -> Waker {
        let raw_waker = RawWaker::new(s as *const (), &VTABLE);
        unsafe { Waker::from_raw(raw_waker) }
    }



