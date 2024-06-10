
//const _SSID: &str = "-_-";
//const _PASSWORD: &str = "asdfmovie";
const _SSID: &str = "Sicheres Muggle-WLAN 2.4";
const _PASSWORD: &str = "Accio-Internet";


use embassy_time::{Duration, Timer};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_net::Stack;
use embedded_io::ReadReady;

use sha1::compute_sha1;
use messages::Message;
use super::client_app;

pub async fn wait_for_wifi_connection(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>){
    loop {
        if stack.is_link_up() {
            log::info!("Link is up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    log::info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            log::info!("Got IP: {}", config.address);
            log::debug!("Gateway: {}", config.gateway.unwrap());
            for dns in config.dns_servers.iter() {
                log::debug!("DNS: {}", dns);
            }
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
pub async fn websocket_server(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>, 
                              control: &'static Signal<CriticalSectionRawMutex, messages::Message>){
    loop{
        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];
        let mut socket = WebSocket::new(embassy_net::tcp::TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer));
    
        log::info!("Waiting for websocket connection...");
        match socket.accept(embassy_net::IpListenEndpoint::from(messages::MESSAGE_PORT)).await {
            Ok(_) => log::info!("Websocket connection established"),
            Err(e) => log::info!("Websocket connection failed: {:?}", e),
        }

        while socket.may_recv() {
            match socket.read().await {
                Ok(msg) => {
                    log::debug!("Websocket recieved message:  {:?}", msg);
                    control.signal(msg);
                }
                Err(e) => {
                    match e{
                        WebSocketError::InboxEmpy => {
                            Timer::after(Duration::from_millis(100)).await;
                        },
                        _ => {
                            log::info!("Websocket read error: {:?}", e);
                            break;
                        },
                    }
                }
            };
        }
    }
}

pub const HTTP_SERVER_POOL_SIZE: usize = 2; 
#[embassy_executor::task(pool_size = HTTP_SERVER_POOL_SIZE)]
pub async fn http_server(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>, task_id: usize) {
    const RX_BUF_LEN: usize = 1024;
    const TX_BUF_LEN: usize = 8192; //the bigger the faster large files will load, should be between 4kb and 32kb for esp32 //Upper limit may be smaller (4kb ~ 250kb/s; 8kb ~ 400kb/s; 32kb  ~ 500 kb/s; 64kb ~ 500 kb/s)
    let mut rx_buffer = [0; RX_BUF_LEN];
    let mut tx_buffer = [0; TX_BUF_LEN];
    loop{
        let mut socket = embassy_net::tcp::TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
        socket.set_keep_alive(Some(Duration::from_millis(200)));
        socket.accept(embassy_net::IpListenEndpoint::from(80u16)).await.unwrap();
        let remote_endpoint = socket.remote_endpoint().unwrap();

        let mut buf = [0; RX_BUF_LEN];
        let request;
        let header;
        match socket.read(&mut buf).await {
            Ok(n) => {
                match core::str::from_utf8(&buf[..n]){
                    Ok(ss) => {
                        if let Some(n) = ss.find("\n"){
                            request = ss.split_at(n).0.trim();
                            header = ss.split_at(n).1.trim();
                        }
                        else{
                            request = ss;
                            header = "Empyt Header\n";
                        }
                    },
                    Err(e) => {
                        log::warn!("(HTTP Server {task_id}) read {n} invalid utf8 bytes: {:?}", e);
                        socket.close();
                        socket.flush().await.unwrap();
                        continue;
                    }
                }
            },
            Err(e) => {
                log::warn!("(HTTP Server {task_id}) read error: {:?}", e);
                socket.close();
                socket.flush().await.unwrap();
                continue;
            }
        };
        log::info!("(HTTP Server {task_id}) received request from {remote_endpoint}: {request}");
        log::debug!("(HTTP Server {task_id}) received header: \n{header}");
        
        let mut file2serve = None;
        for file in client_app::CLIENT_APP_FILES.iter(){
            if request.contains(file.0){
                file2serve = Some(*file);
                break;
            }
        }
        
        if let Some(f) = file2serve{
            log::info!("(HTTP Server {task_id}) Serving {}", f.0);
            match f.0.split(".").last(){
                Some("js") => socket.write(b"HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\n\r\n").await.unwrap(),
                Some("wasm") => socket.write(b"HTTP/1.1 200 OK\r\nContent-Type: application/wasm\r\n\r\n").await.unwrap(),
                _ => socket.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n").await.unwrap(),
            };
            tcp_write_large(&mut socket, f.1).await.unwrap();
        }
        else if request.contains("GET / HTTP/1.1") || request.contains("GET /index.html HTTP/1.1"){
            log::info!("(HTTP Server {task_id}) Serving index.html");
            socket.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n").await.unwrap();
            tcp_write_large(&mut socket, client_app::INDEX_HTML).await.unwrap();
        }
        else{
            log::info!("(HTTP Server {task_id}) Serving 404");
            socket.write(b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\n404 Not Found").await.unwrap();
        }

        socket.close();
        socket.flush().await.unwrap();
        log::info!("(HTTP Server {task_id}) connection closed\n");
    }
}

async fn tcp_write_large(socket: &mut embassy_net::tcp::TcpSocket<'_>, data: &[u8]) -> Result<(), embassy_net::tcp::Error> {
    let mut sent = 0;
    while sent < data.len() {
        let len;
        if sent + socket.send_capacity() < data.len() {
            len = socket.send_capacity();
        } else {
            len = data.len() - sent;
        }
        let n = socket.write(&data[sent..(sent+len)]).await?;
        sent += n;
    }
    Ok(())
}

#[embassy_executor::task]
pub async fn wifi_connection(mut controller: esp_wifi::wifi::WifiController<'static>) {
    use esp_wifi::wifi::{WifiState, WifiEvent, Configuration, ClientConfiguration};
    use esp_println::println;

    println!("connection task started");
    println!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                log::info!("Wifi connected :)");
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: _SSID.try_into().unwrap(),
                password: _PASSWORD.try_into().unwrap(),
                auth_method: esp_wifi::wifi::AuthMethod::WPAWPA2Personal,
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>) {
    stack.run().await
}



const _WS_ANSWER_: &[u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ";
const _WS_ANSWER: &str = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ";
const _WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
enum WebSocketError {
    InvalidState,
    InvalidPort,
    ConnectionReset,
    InvalidMessage,
    InboxEmpy,
}

impl From<embassy_net::tcp::AcceptError> for WebSocketError {
    fn from(e: embassy_net::tcp::AcceptError) -> Self {
        match e {
            embassy_net::tcp::AcceptError::ConnectionReset => WebSocketError::ConnectionReset,
            embassy_net::tcp::AcceptError::InvalidState => WebSocketError::InvalidState,
            embassy_net::tcp::AcceptError::InvalidPort => WebSocketError::InvalidPort,
        }
    }
}

impl From<embassy_net::tcp::Error> for WebSocketError {
    fn from(e: embassy_net::tcp::Error) -> Self {
        match e {
            embassy_net::tcp::Error::ConnectionReset => WebSocketError::ConnectionReset,
        }
    }
}

struct WebSocket<'a>{
    tcp_socket: embassy_net::tcp::TcpSocket<'a>,
}

impl<'a> WebSocket<'a>{
    pub fn new(tcp_socket: embassy_net::tcp::TcpSocket<'a>) -> Self{
        let mut s = Self{
            tcp_socket,
        };
        s.tcp_socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
        s.tcp_socket.set_keep_alive(Some(Duration::from_millis(200)));
        s
    }
    
    pub async fn accept(&mut self, endpoint: embassy_net::IpListenEndpoint) -> Result<(), WebSocketError>{
        let mut buf = [0; 1024];
        let mut data = heapless::String::<1024>::new();
        let mut to_be_hashed = heapless::String::<64>::new();
        let hash;
        let mut response = heapless::String::<142>::new();

        self.tcp_socket.accept(endpoint).await?;
        self.tcp_socket.read(&mut buf).await?;

        data.push_str(core::str::from_utf8(&buf).unwrap()).unwrap();
        for line in data.lines(){
            if line.starts_with("Sec-WebSocket-Key:"){
                to_be_hashed.push_str(line.split(":").nth(1).unwrap().trim()).unwrap();
                to_be_hashed.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11").unwrap();
                break;
            }
        }
        hash = compute_sha1(to_be_hashed.as_bytes());
        response.push_str(_WS_ANSWER).unwrap();
        response.push_str(&sha1::sha1_bytes_2_base64(&hash)).unwrap();
        response.push_str("\r\n\r\n").unwrap();
        self.tcp_socket.write(response.as_bytes()).await?;

        Ok(())
    }

    pub fn may_recv(&self) -> bool{
        self.tcp_socket.may_recv()
    }

    pub async fn read(&mut self) -> Result<Message, WebSocketError>{
        if self.tcp_socket.read_ready()?{
            let mut buf = [0; 1024];
            let n = self.tcp_socket.read(&mut buf).await?;
            let mut data = heapless::String::<1024>::new();
            let frame_type = Self::unpack_clientframe(&mut buf[..n]);
            match frame_type{
                WebSocketFrameType::Text(m) => {
                    data.push_str(core::str::from_utf8(&buf[..m]).unwrap()).unwrap();
                    let msg = Message::from_str(data.as_str());
                    if let Some(m) = msg{
                        return Ok(m);
                    }
                    else{
                        log::warn!("unrecognized message: {data}");
                        return Err(WebSocketError::InvalidMessage);
                    }
                }
                WebSocketFrameType::Binary(m) => {
                    log::warn!("{m} Bytes binary data recieved but not supported");
                    Err(WebSocketError::InvalidMessage)
                }
                _ => {
                    log::warn!("Not supported frame type: {frame_type:?}");
                    Err(WebSocketError::InvalidMessage)
                }
            }
        }
        else{
            return Err(WebSocketError::InboxEmpy);
        }
    }

    fn unpack_clientframe(data: &mut [u8]) -> WebSocketFrameType{
        let payload_len: u64;
        let payload_start: usize;
        let mut mask_key = [0u8; 4];
        let frame_type: WebSocketFrameType;
        if data.len() < 2{
            log::warn!("Data is too short (2) for websocket frame");
            return WebSocketFrameType::Invalid;
        }
        if data[0] & 0x80 != 0x80{
            log::warn!("Not a final websocket frame");
            return WebSocketFrameType::Invalid;
        }
        if data[0] & 0b01110000 != 0{
            log::warn!("websocket frame uses unknown extension");
            return WebSocketFrameType::Invalid;
        }
        if data[1] & 0x80 != 0x80{
            log::warn!("Mask bit not set");
            return WebSocketFrameType::Invalid;
        }
        match data[1] & 0b01111111{//get payload length and start
            0x7E => {
                if data.len() < 8{
                    log::warn!("Data is too short (8) for websocket frame");
                    return WebSocketFrameType::Invalid;
                }
                payload_len = u64::from_be_bytes(data[2..4].try_into().unwrap());
                payload_start = 8;
                for i in 0..4{
                    mask_key[i] = data[i+4];
                }
            }
            0x7F => {
                if data.len() < 14{
                    log::warn!("Data is too short (14) for websocket frame");
                    return WebSocketFrameType::Invalid;
                }
                payload_len = u64::from_be_bytes(data[2..10].try_into().unwrap());
                payload_start = 14;
                for i in 0..4{
                    mask_key[i] = data[i+10];
                }
            }
            _ => {
                if data.len() < 6{
                    log::warn!("Data is too short (6) for websocket frame");
                    return WebSocketFrameType::Invalid;
                }
                payload_len = (data[1] & 0b01111111) as u64;
                payload_start = 6;
                for i in 0..4{
                    mask_key[i] = data[i+2];
                }
            }
        }
        if payload_len > (data.len() - payload_start) as u64{
            log::warn!("Payload length is longer than data");
            return WebSocketFrameType::Invalid;
        }
        match data[0] & 0b1111{
            0x0 => frame_type = WebSocketFrameType::Continuation,
            0x1 => frame_type = WebSocketFrameType::Text(payload_len as usize),
            0x2 => frame_type = WebSocketFrameType::Binary(payload_len as usize),
            0x8 => frame_type = WebSocketFrameType::ConnectionClose,
            0x9 => frame_type = WebSocketFrameType::Ping,
            0xA => frame_type = WebSocketFrameType::Pong,
            _   => frame_type = WebSocketFrameType::Unknown,
        }
        for i in payload_start..data.len(){
            data[i-payload_start] = data[i] ^ mask_key[(i-payload_start) % 4];
        }
        frame_type
    }
}

#[derive(Debug)]
enum WebSocketFrameType{
    Continuation,
    Text(usize),
    Binary(usize),
    ConnectionClose,
    Ping,
    Pong,
    Unknown,
    Invalid,
}
