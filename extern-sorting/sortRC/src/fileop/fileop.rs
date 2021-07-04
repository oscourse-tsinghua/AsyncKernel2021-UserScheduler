use std::{fs::OpenOptions, io::Write};


#[allow(unused_must_use)]
pub fn write_vec(nums:Vec<u32>,file_name:String){
    let mut file_write = OpenOptions::new().append(true).open(file_name).expect("cannot open file");
    for i in 0..nums.len() {
        file_write.write((nums[i].to_string() + &" ".to_string()).as_bytes());
        if (i + 1) % 100 == 0 {
            file_write.write(("\n".to_string()).as_bytes()); 
        }
    }
}

// string => vec
pub fn string_to_vec(str:String) -> Vec<u32> {
    let mut res = vec![];
    let mut lines = str.lines();
    loop{
        let line = lines.next();
        // 如果下一行为空，说明已读完
        if line == None {break;}
        let mut res_temp = line_to_vec(Some(line.unwrap().to_string()));
        res.append(&mut res_temp);
    }
    return res;
}

// 将一行字符串数字转换为vec
pub fn line_to_vec(line : Option<String> ) -> Vec<u32> {
    let mut res = vec![];
        match line {
            
            None => {
                
            }
            Some(str) => {
                // 按空格拆分字符串数字
                let mut nums_str = str.split_ascii_whitespace();
                loop{
                    let next = nums_str.next();
                    match next {
                        None => break,

                        Some(num_str)=> {
                            
                            let num:u32 = num_str.to_string().parse().unwrap();
                            res.push(num);
                        }

                    }
                }
            }
        }
    
    return res;
}
