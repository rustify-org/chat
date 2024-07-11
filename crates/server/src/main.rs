use std::io::{ErrorKind, Read, Write}; // 导入输入输出相关的库
use std::net::TcpListener; // 导入TCP监听器
use std::sync::mpsc; // 导入多线程消息传递
use std::time::Duration; // 导入时间处理
use std::{thread, vec}; // 导入线程和向量

const LOCAL_SERVER_PORT: &str = "127.0.0.1:8888"; // 定义服务器地址
const MSG_SIZE: usize = 1024; // 定义消息大小

fn main() {
    // 绑定到指定的地址和端口
    let server = TcpListener::bind(LOCAL_SERVER_PORT).expect("Failed to bind");

    // 设置为非阻塞模式
    server
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let mut clients: Vec<_> = vec![]; // 创建一个向量用于存储客户端
    let (tx, rx) = mpsc::channel::<String>(); // 创建一个多线程通道用于消息传递

    loop {
        // 监听新的连接
        if let Ok((mut socket, addr)) = server.accept() {
            println!("{} connected", addr); // 打印连接的客户端地址
            clients.push(socket.try_clone().expect("Failed to clone")); // 克隆socket并存储到客户端向量中
            let tx = tx.clone(); // 克隆发送端
                                 // 创建一个新线程处理客户端消息
            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE]; // 创建一个缓冲区
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        // 接收消息并转换为字符串
                        let msg: Vec<u8> =
                            buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg_string = String::from_utf8(msg).expect("Failed to convert");
                        println!("{}: {:?}", addr, msg_string); // 打印消息
                        tx.send(msg_string).expect("Failed to send"); // 发送消息到通道
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        // 处理断开连接
                        println!("{} disconnected", addr);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(100)); // 休眠100毫秒
            });
        }
        // 转发消息给所有客户端
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes(); // 转换消息为字节数组
                    buff.resize(MSG_SIZE, 0); // 调整缓冲区大小
                    client.write_all(&buff).map(|_| client).ok() // 发送消息
                })
                .collect::<Vec<_>>() // 更新客户端列表
        }
        thread::sleep(Duration::from_micros(100)); // 休眠100微秒
    }
}
