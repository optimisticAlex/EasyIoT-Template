
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
/* 
#[embassy_executor::task]
pub async fn websocket_echo_server(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>) {
    use heapless::String;
    loop{
        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];
        let mut socket = embassy_net::tcp::TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
        socket.set_keep_alive(Some(Duration::from_millis(200)));

        log::info!("Echo server waiting for connection...");
        socket.accept(embassy_net::IpListenEndpoint::from(420u16)).await.unwrap();
        log::info!("Accepted websocket connection from {}", socket.remote_endpoint().unwrap());

        //Read data from Socket
        let mut data = String::<1024>::new();
        let mut buf = [0; 1024];
        match socket.read(&mut buf).await {
            Ok(n) => {
                log::info!("Read {n} bytes: {}", core::str::from_utf8(&buf[..n]).unwrap());
            }
            Err(e) => {
                log::info!("read error: {:?}", e);
                continue;
            }
        };
        //Get Sec-WebSocket-Key and append "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
        data.push_str(core::str::from_utf8(&buf).unwrap()).unwrap();
        let mut to_be_hashed = String::<64>::new();
        for line in data.lines(){
            if line.starts_with("Sec-WebSocket-Key:"){
                to_be_hashed.push_str(line.split(":").nth(1).unwrap().trim()).unwrap();
                to_be_hashed.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11").unwrap();
                break;
            }
        }
        let to_be_hashed_len = to_be_hashed.len();
        esp_println::println!("to_be_hashed({to_be_hashed_len}): {to_be_hashed}");
        //hash with sha1
        let hashed = compute_sha1(to_be_hashed.as_bytes());
        esp_println::println!("Hashed: {}", sha1::sha1_bytes_2_str(&hashed));
        //base64 encode
        esp_println::println!("Encoded: {}", sha1::sha1_bytes_2_base64(&hashed));
        //send back to client
        let mut response = String::<142>::new();
        for byte in _WS_ANSWER_.iter(){
            response.push(*byte as char).unwrap();
        }
        response.push_str(&sha1::sha1_bytes_2_base64(&hashed)).unwrap();
        response.push_str("\r\n\r\n").unwrap();
        socket.write(response.as_bytes()).await.unwrap();

        while socket.may_recv() {
            if socket.read_ready().unwrap(){
                match socket.read(&mut buf).await {
                    Ok(n) => {
                        log::info!("Websocket server recieved {n} bytes");
                        if let Some(m) = websocket_unpack_clientframe(&mut buf[..n]){
                            log::info!("{}", core::str::from_utf8(&buf[..m]).unwrap());
                        }
                        else{
                            break;
                        }
                    }
                    Err(e) => {
                        log::info!("read error: {:?}", e);
                        continue;
                    }
                };
            }
            Timer::after(Duration::from_millis(100)).await;
        }
        socket.close();
        socket.flush().await.unwrap();
        log::info!("Remote Host closed connection\n");
    }
}

fn websocket_unpack_clientframe(data: &mut[u8]) -> Option<usize>{
    if data.len() < 10{
        log::warn!("Data is too short for websocket frame");
        return None;
    }
    let payload_len: u64;
    let payload_start: usize;
    let mut mask_key = [0u8; 4];
    if data[0] & 0x80 != 0x80{
        log::warn!("Not a final websocket frame");
    }
    if data[0] & 0b01110000 != 0{
        log::warn!("websocket frame uses unknown extension");
        return None;
    }
    match data[0] & 0b1111{
        0x0 => log::info!("Continuation frame"),
        0x1 => log::info!("Text frame"),
        0x2 => log::info!("Binary frame"),
        0x8 => log::info!("Connection close frame"),
        0x9 => log::info!("Ping frame"),
        0xA => log::info!("Pong frame"),
        _ => {
            log::warn!("Unknown frame type");
            return None;
        }
    }
    if data[1] & 0x80 != 0x80{
        log::warn!("Mask bit not set");
        return None;
    }
    match data[1] & 0b01111111{
        0x7E => {
            payload_len = u64::from_be_bytes(data[2..4].try_into().unwrap());
            payload_start = 8;
            for i in 0..4{
                mask_key[i] = data[i+4];
            }
        }
        0x7F => {
            payload_len = u64::from_be_bytes(data[2..10].try_into().unwrap());
            payload_start = 14;
            for i in 0..4{
                mask_key[i] = data[i+10];
            }
        }
        _ => {
            payload_len = (data[1] & 0b01111111) as u64;
            payload_start = 6;
            for i in 0..4{
                mask_key[i] = data[i+2];
            }
        }
    }
    if payload_len > (data.len() - payload_start) as u64{
        log::warn!("Payload length is longer than data");
        return None;
    }
    for i in payload_start..data.len(){
        data[i-payload_start] = data[i] ^ mask_key[(i-payload_start) % 4];
    }
    Some(payload_len as usize)
}
 */
#[embassy_executor::task]
pub async fn websocket_server(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>, 
                              control: &'static Signal<CriticalSectionRawMutex, messages::Message>){
    loop{
        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];
        let mut socket = WebSocket::new(embassy_net::tcp::TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer));
    
        log::info!("Waiting for websocket connection...");
        match socket.accept(embassy_net::IpListenEndpoint::from(420u16)).await {
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

#[embassy_executor::task]
pub async fn run_http_server(stack: &'static Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>) {
    loop{
        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];
        let mut socket = embassy_net::tcp::TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
        socket.set_keep_alive(Some(Duration::from_millis(200)));

        log::info!("Waiting for http request...");
        socket.accept(embassy_net::IpListenEndpoint::from(80u16)).await.unwrap();
        log::info!("Accepted http request from {}", socket.remote_endpoint().unwrap());

        let mut buf = [0; 1024];
        match socket.read(&mut buf).await {
            Ok(n) => {
                log::info!("Read {n} bytes: {}", core::str::from_utf8(&buf[..n]).unwrap());
            }
            Err(e) => {
                log::info!("read error: {:?}", e);
                continue;
            }
        };

        socket.write(b"ToDo").await.unwrap();
        socket.close();
        socket.flush().await.unwrap();
        log::info!("Waiting for Remote Host to close connection...");
        while socket.may_recv() {
            Timer::after(Duration::from_millis(1000)).await;
        }
        log::info!("Remote Host closed connection\n");
    }
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
