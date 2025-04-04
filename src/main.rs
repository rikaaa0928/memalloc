use std::env;
use std::process;
use std::thread;

fn parse_size(size_str: &str) -> Option<usize> {
    let lower_str = size_str.to_lowercase();
    // 处理纯数字的情况
    if lower_str.chars().all(|c| c.is_digit(10)) {
        return lower_str.parse().ok();
    }

    // 找到第一个非数字字符的位置作为单位的开始
    let split_index = lower_str.find(|c: char| !c.is_digit(10));
    let (num_str, unit) = match split_index {
        Some(index) => lower_str.split_at(index),
        None => return None, // 如果没有单位且不是纯数字，则格式无效
    };

    let num: usize = match num_str.parse() {
        Ok(n) => n,
        Err(_) => return None,
    };

    match unit.trim() { // 去除单位前后的空格
        "" | "b" => Some(num), // 允许没有单位或'b'表示字节
        "kb" | "k" => Some(num.saturating_mul(1024)),
        "mb" | "m" => Some(num.saturating_mul(1024 * 1024)),
        "gb" | "g" => Some(num.saturating_mul(1024 * 1024 * 1024)),
        _ => None, // 未知单位
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("用法: {} <内存大小>", args[0]);
        eprintln!("示例: {} 100mb", args[0]);
        eprintln!("支持的单位: b (默认), kb (k), mb (m), gb (g)");
        process::exit(1);
    }

    let size_str = &args[1];
    let size_bytes = match parse_size(size_str) {
        Some(size) if size > 0 => size, // 确保分配大小大于0
        Some(_) => { // 处理 size = 0 的情况
             eprintln!("错误: 分配的内存大小必须大于 0 字节。");
             process::exit(1);
        }
        None => {
            eprintln!("错误: 无法解析内存大小 '{}'", size_str);
            eprintln!("请使用数字和可选单位 (b, kb, mb, gb)，例如: 512mb, 1gb, 2048");
            process::exit(1);
        }
    };

    println!("尝试分配 {} 字节内存...", size_bytes);

    // 使用 Vec<u8> 分配内存
    let mut memory_holder: Vec<u8> = Vec::new();
    // 使用 try_reserve_exact 尝试精确预留内存
    if let Err(e) = memory_holder.try_reserve_exact(size_bytes) {
         eprintln!("错误: 无法预留 {} 字节内存: {}", size_bytes, e);
         // 提示可能是系统内存不足
         eprintln!("这可能是因为系统内存不足或请求的大小过大。");
         process::exit(1);
    }

    // 使用 resize 来实际分配并初始化内存（通常为零初始化）
    // 这确保操作系统实际将内存页分配给进程
    memory_holder.resize(size_bytes, 0);

    // 再次检查以确认 resize 是否成功（虽然 resize 本身不直接返回 Result，但如果失败会 panic）
    // 检查 len() 是否等于请求的大小
    if memory_holder.len() != size_bytes {
        eprintln!("错误: 分配后的内存大小 ({}) 与请求的大小 ({}) 不符。", memory_holder.len(), size_bytes);
        process::exit(1);
    }


    // 打印成功信息
    println!("成功分配并初始化了 {} 字节内存。", memory_holder.len());
    println!("内存已分配。程序将保持运行状态。");
    println!("按 Ctrl+C 或使用 kill 命令终止进程。");

    // 保持进程运行，直到被外部信号终止
    // 使用 park() 可以让线程休眠，直到被 unpark() 或伪信号唤醒，比 sleep 更节能
    loop {
        thread::park(); // 阻塞主线程，等待被终止
    }
}
