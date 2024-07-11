use std::io::{self, ErrorKind, Read, Write}; // 导入输入输出相关的库
use std::net::TcpStream; // 导入TCP流
use std::sync::mpsc::{self, TryRecvError}; // 导入多线程消息传递
use std::time::Duration; // 导入时间处理
use std::{thread, vec}; // 导入线程和向量

const LOCAL_SERVER: &str = "127.0.0.1:8888"; // 定义服务器地址
const MSG_SIZE: usize = 1024; // 定义消息大小

fn main() {
    // 连接到服务器
    let mut client = TcpStream::connect(LOCAL_SERVER).expect("Failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let (tx, rx) = mpsc::channel::<String>(); // 创建一个多线程通道用于消息传递

    // 创建一个新线程处理接收和发送消息
    std::thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE]; // 创建一个缓冲区
                                          // 读取服务器消息
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg_string = String::from_utf8(msg.clone()).unwrap();
                println!("{}", msg_string); // 打印消息
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                // 处理断开连接
                println!(">>Client closed");
                break;
            }
        }
        // 发送消息到服务器
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes(); // 转换消息为字节数组
                buff.resize(MSG_SIZE, 0); // 调整缓冲区大小
                client.write_all(&buff).expect("Failed to send"); // 发送消息
                println!(">>{}", msg)
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }
        thread::sleep(Duration::from_millis(100)); // 休眠100毫秒
    });
    println!(">>Enter your message");
    loop {
        let mut buff = String::new(); // 创建一个缓冲区用于存储用户输入
        io::stdin().read_line(&mut buff).expect("Failed to read"); // 读取用户输入
        let msg = buff.trim().to_string(); // 去除多余的空格
        if msg == "quit" || tx.send(msg).is_err() {
            // 如果输入"quit"或通道关闭，则退出循环
            break;
        }
    }
    println!(">>Client closed");
}
