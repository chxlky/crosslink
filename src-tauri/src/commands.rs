use std::net::Ipv4Addr;
use std::time::Duration;
use std::{net::UdpSocket, thread};

use sysinfo::{Pid, System};

use crate::structs::Device;

#[tauri::command]
pub fn discover_devices() -> Vec<Device> {
    thread::spawn(|| {
        let socket = UdpSocket::bind("0.0.0.0:34254").expect("Could not bind to address");
        socket.set_broadcast(true).expect("Could not set broadcast");

        loop {
            let mut buf = [0; 10];
            let (amt, src) = socket.recv_from(&mut buf).expect("Could not receive data");

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
    });

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
    Ok(())
}
