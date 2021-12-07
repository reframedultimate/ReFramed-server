use std::sync::Mutex;
use std::vec::Vec;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::protocol;

pub struct Server {
    clients: Mutex<Vec<mpsc::Sender<Vec<u8>>>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: Mutex::new(Vec::new())
        }
    }

    pub fn accept_client(&self, mut client: TcpStream) {
        match client.set_nonblocking(true) {
            Ok(()) => (),
            Err(e) => {
                println!("[uhrecorder] Failed to set nonblocking: {}", e);
                return;
            }
        }

        match client.local_addr() {
            Ok(addr) => println!("[uhrecorder] Accepting client connection {}", addr),
            Err(_) => println!("[uhrecorder] Accepting client connection")
        }

        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            println!("[uhrecorder] Started read thread");
            loop {
                let mut buf = vec![];
                match client.read(&mut buf) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("[uhrecorder] read() failed, exiting listen thread: {}", e);
                        break;
                    }
                }

                match rx.recv() {
                    Ok(data) => match client.write(&data) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("[uhrecorder] write() failed, exiting listen thread: {}", e);
                            break;
                        }
                    },
                    Err(e) => {
                        println!("[uhrecorder] recv() failed, exiting listen thread: {}", e);
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(4));
            }
        });

        match protocol::send_constants(&tx) {
            Ok(()) => println!("[uhrecorder] Sent constants"),
            Err(e) => {
                println!("[uhrecorder] Failed to send constants: {}", e);
                return;
            }
        }

        match self.clients.lock() {
            Ok(mut clients) => clients.push(tx),
            Err(e) => {
                println!("[uhrecorder] Poisoned mutex: {}", e);
                return;
            }
        }
    }

    pub fn listen_for_incoming_connections(&self) {
        let listener = match TcpListener::bind("0.0.0.0:42069") {
            Ok(listener) => listener,
            Err(e) => { println!("[uhrecorder] Failed to bind socket: {}", e); return; }
        };

        match listener.local_addr() {
            Ok(addr) => println!("[uhrecorder] Started server on {}", addr),
            Err(_) => println!("[uhrecorder] Started server")
        };

        for stream in listener.incoming() {
            match stream {
                Ok(client) => self.accept_client(client),
                Err(e) => {
                    println!("[uhrecorder] Accept failed: {}", e);
                    break;
                }
            }
        }
    }

    pub fn broadcast(&self, data: &[u8]) {
        match self.clients.lock() {
            Ok(mut clients) => clients.retain(|ref tx| {
                match tx.send(data.to_vec()) {
                    Ok(_) => true,
                    Err(e) => {
                        println!("[uhrecorder] send() failed, removing client: {}", e);
                        false
                    }
                }
            }),
            Err(_) => println!("[uhrecorder] Lock poisoned")
        }
    }
}

