#![feature(llvm_asm, naked_functions)]
#![feature(impl_trait_in_bindings)]

use kernel_task::KernelThread;
use runtime::{Runtime};

use crate::kernel_task::Task;

mod kernel_task;
mod runtime;


fn main() {

    // 初始化运行时：任务队列 + 执行器
    let mut rt = Runtime::new();
    rt.init();

    println!("runtime init");

    KernelThread::new(||{
        println!("t1, as line 18");

        Task::new(||{
            println!("t1's task1, at line 23");
        });
        Task::new(||{
            println!("t1's task2, at line 26");
        });

        println!("t1, at line 29, exit");
    });

    KernelThread::new(||{
        println!("t2, at line 34");

        Task::new(||{
            println!("t2's task1, at line 37");
        });

        println!("t2, at line 40, exit");
    });

    rt.run();
}






