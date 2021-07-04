pub mod fileop{
    pub mod fileop;
}
use fileop::fileop::*;

use std::{fs, io::Read, thread, time::Instant};

// 48M : 50331648
// 44M : 46137344
// 40M : 41943040
// 36M : 37748736
// 32M : 33554432
// 24M : 25165824
// 16M : 16777216
// 12M : 12582912
// 8M  : 8388608
// 4M  : 4194304
// 1M  : 1048576
// 512k: 524288
const BUFFER_SIZE:usize = 4194304;

fn main() {
    // 开启计时
    let start = Instant::now();
    // 单线程读
    let t1 = thread::spawn(move || {
        let mut i = 1;
        println!("t1 read & sort & write start {}",start.elapsed().as_secs_f64());
        // 打开文件
        let file_name = "sort1.txt".to_string();
        let mut file = fs::File::open(file_name).unwrap();
        // 设置缓冲区
        static mut buffer: [u8; BUFFER_SIZE] = [32u8;BUFFER_SIZE];
        unsafe {
            loop {
                i += 1;
                buffer = [32u8;BUFFER_SIZE];
                // 读文件
                println!("read sort1.txt start  {}",start.elapsed().as_secs_f64());
                let len = file.read(&mut buffer).unwrap();
                if len == 0 {break;}
                println!("read sort1.txt finish {}",start.elapsed().as_secs_f64());
                // string => vec
                let result = String::from_utf8(buffer.to_vec()).unwrap();
                //let mut nums = string_to_vec(result);
                // 排序
                //nums.sort();
                //let write_file = "data1_".to_string() + &(i % 2 + 1).to_string() + &".txt".to_string();
                //write_vec(nums, write_file);
            }
            println!("t1 read &  write finish {}",start.elapsed().as_secs_f64());
        }

        /*
        // merge
        println!("t1 merge read start {}",start.elapsed().as_secs_f64());
        let mut result = "".to_string();
        for i in 0..2 {
            let count = (i+1).to_string().clone();
            let file_name = "data1_".to_string() + &(i+1).to_string() + &".txt".to_string(); 

            let result_no_reactor = read_file_no_reactor(file_name);

            result.push_str(&result_no_reactor);
        }
        println!("t1 merge read finish {}",start.elapsed().as_secs_f64());
        // string to vec
        let mut nums = string_to_vec(result);
        // 排序
        nums.sort();
        // 写文件
        let write_file = "result1.txt".to_string();
        write_vec(nums, write_file);
        */
    });

    let t2 = thread::spawn(move || {
        let mut i = 1;
        println!("t2 read & sort & write start {}",start.elapsed().as_secs_f64());
    
        // 打开文件
        let file_name = "sort2.txt".to_string();
        let mut file = fs::File::open(file_name).unwrap();
        // 设置缓冲区
        static mut buffer: [u8; BUFFER_SIZE] = [32u8;BUFFER_SIZE];

        unsafe {
            loop {
                i += 1;
                buffer = [32u8;BUFFER_SIZE];
                println!("read sort2.txt start  {}",start.elapsed().as_secs_f64());
                // 读文件
                let len = file.read(&mut buffer).unwrap();
                if len == 0 {break;}
                println!("read sort2.txt finish {}",start.elapsed().as_secs_f64());
                // string => vec
                let result = String::from_utf8(buffer.to_vec()).unwrap();
                //let mut nums = string_to_vec(result);
                // 排序
                //nums.sort();
                //let write_file = "data2_".to_string() + &(i % 5 + 1).to_string() + &".txt".to_string();
                //write_vec(nums, write_file);
            }
            println!("t2 read & sort & write finish {}",start.elapsed().as_secs_f64());    

        }
        /*
        // merge
        println!("t2 merge read start {}",start.elapsed().as_secs_f64());
        let mut result = "".to_string();
        for i in 0..5 {
            let count = (i+1).to_string().clone();
            let file_name = "data2_".to_string() + &(i+1).to_string() + &".txt".to_string(); 

            let result_no_reactor = read_file_no_reactor(file_name);

            result.push_str(&result_no_reactor);
        }
        println!("t2 merge read finish {}",start.elapsed().as_secs_f64());
        // string to vec
        let mut nums = string_to_vec(result);
        // 排序
        nums.sort();
        // 写文件
        let write_file = "result2.txt".to_string();
        write_vec(nums, write_file);
        */
    });

    t1.join();
    t2.join();
}

// 直接读文件
#[allow(dead_code)]
fn read_file_no_reactor(file_name:String) -> String {
    let buf = fs::read_to_string(file_name).unwrap();
    buf
}
