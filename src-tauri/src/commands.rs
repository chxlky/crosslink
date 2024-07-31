use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use std::{net::UdpSocket, thread};

use sysinfo::{Pid, System};
use tauri::Emitter;

use crate::structs::Device;

lazy_static::lazy_static! {
    static ref SOCKET: Arc<RwLock<Option<UdpSocket>>> = Arc::new(RwLock::new(None));
}

#[tauri::command]
pub fn discover_devices() -> Vec<Device> {
    {
        let mut socket_guard = SOCKET.write().unwrap();
        if socket_guard.is_none() {
            let socket = UdpSocket::bind("0.0.0.0:34254").expect("Could not bind to address");
            socket.set_broadcast(true).expect("Could not set broadcast");
            *socket_guard = Some(socket);

            let socket_clone = Arc::clone(&SOCKET);
            thread::spawn(move || {
                let socket = socket_clone.read().unwrap();
                if let Some(ref socket) = *socket {
                    loop {
                        let mut buf = [0; 10];
                        let (amt, src) =
                            socket.recv_from(&mut buf).expect("Could not receive data");

                        if &buf[..amt] == b"DISCOVER" {
                            let mut sys = System::new_all();
                            sys.refresh_all();
                            let pid = Pid::from_u32(std::process::id());

                            if let Some(_process) = sys.process(pid) {
                                let os = System::name().unwrap_or("Unknown".to_string());
                                let hostname = System::host_name().unwrap_or("Unknown".to_string());
                                let response = format!("NAME:{}\nOS:{}", hostname, os);

                                socket
                                    .send_to(response.as_bytes(), src)
                                    .expect("Could not send data");
                            }
                        }
                    }
                }
            });
        }
    }

    thread::sleep(Duration::from_secs(1));

    let socket = UdpSocket::bind("0.0.0.0:34255").expect("Could not bind to address");
    socket.set_broadcast(true).expect("Could not set broadcast");
    socket
        .send_to(b"DISCOVER", (Ipv4Addr::BROADCAST, 34254))
        .expect("Could not send data");

    let mut buf = [0; 1024];
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("Could not set read timeout");

    let mut devices = Vec::new();

    let current_device_name = System::host_name().unwrap_or("Unknown".to_string());

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let response = String::from_utf8_lossy(&buf[..amt]);
                let (name, os) = parse_name_and_os(&response.to_string());

                if name != current_device_name {
                    devices.push(Device {
                        ip: src.ip().to_string(),
                        hostname: name,
                        os,
                    })
                }
            }
            Err(_) => break,
        }
    }

    devices
}

fn parse_name_and_os(response: &str) -> (String, String) {
    let mut name = "Unknown".to_string();
    let mut os = "Unknown".to_string();

    for line in response.lines() {
        if line.starts_with("NAME:") {
            name = line.trim_start_matches("NAME:").trim().to_string();
        } else if line.starts_with("OS:") {
            os = line.trim_start_matches("OS:").trim().to_string();
        }
    }

    (name, os)
}

#[tauri::command]
pub fn send_file(device: Device, file_path: String) -> Result<(), String> {
	println!("Sending file {} to {} at {}", file_path, device.hostname, device.ip);
	let mut stream = TcpStream::connect(format!("{}:7878", &device.ip)).map_err(|e| e.to_string())?;
	let mut file = File::open(file_path).map_err(|e| e.to_string())?;

	let mut buffer = [0; 1024];
	loop {
		let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
		if bytes_read == 0 {
			break;
		}
		stream.write_all(&buffer[..bytes_read]).map_err(|e| e.to_string())?;
	}

    Ok(())
}

#[tauri::command]
pub fn start_file_server(window: tauri::Window) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:7878").expect("Could not bind to address");
        println!("Server listening on port 7878");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tx = tx.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, tx) {
                            eprintln!("Failed to handle client: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });

    thread::spawn(move || {
        while let Ok(file_info) = rx.recv() {
            window.emit("file-received", file_info).unwrap();
        }
    });

    Ok(())
}

fn handle_client(mut stream: TcpStream, tx: Sender<String>) -> io::Result<()> {
    let mut length_buffer = [0; 4];
    stream.read_exact(&mut length_buffer)?;
    let name_length = u32::from_be_bytes(length_buffer) as usize;

    let mut name_buffer = vec![0; name_length];
    stream.read_exact(&mut name_buffer)?;
    let file_name = String::from_utf8(name_buffer).expect("Invalid UTF-8");

	println!("Receiving file {}", &file_name);
    tx.send(file_name).expect("Failed to send file name to frontend");

    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
		println!("Received {} bytes", bytes_read);
    }
	println!("File received successfully");
	Ok(())
}
