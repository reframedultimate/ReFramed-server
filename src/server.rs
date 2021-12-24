use std::vec::Vec;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

use crate::protocol;

pub struct Server {
    clients: Mutex<
        Vec<
            mpsc::Sender<Vec<u8>>
        >
    >
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: Mutex::new(Vec::new())
        }
    }

    pub fn accept_client(&self, client_stream: TcpStream) {
        // Currently we poll the socket because try_clone() doesn't work
        match client_stream.set_nonblocking(true) {
            Ok(()) => (),
            Err(e) => {
                println!("[ReFramed] Failed to set nonblocking: {}", e);
                return;
            }
        }

        // Print the address if possible
        match client_stream.local_addr() {
            Ok(addr) => println!("[ReFramed] Accepting client connection {}", addr),
            Err(_) => println!("[ReFramed] Accepting client connection")
        }

        // Because try_clone() doesn't work, we move the whole socket into
        // a new thread, and as a work-around, write to the ends of
        // a fifo (since those can be split up between threads)
        let (send_tx, send_rx) = mpsc::channel::<Vec<u8>>();
        Server::start_client_thread(client_stream, send_rx);
        self.clients.lock().unwrap().push(send_tx);
    }

    pub fn start_client_thread(mut stream: TcpStream, rx: mpsc::Receiver<Vec<u8>>) {
        // Client thread. Poll socket for new data and process it. Poll
        // send_rx and forward it to the socket so the server thread
        // can send data.
        thread::spawn(move || {
            println!("[ReFramed] Started client thread");
            let mut buf: [u8; 32] = [0; 32];
            loop {

                // Process any incoming data
                let data_was_received = match stream.read(&mut buf) {
                    Ok(size) => {
                        if size == 0 {
                            println!("[ReFramed] socket read() returned 0, exiting client thread");
                            break;
                        }
    
                        match protocol::recv_data(&buf, size, &stream) {
                            Ok(_) => (),
                            Err(_) => {
                                println!("[ReFramed] Receiving end of send channel was closed, exiting client thread");
                                break;
                            }
                        };
                        true
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.raw_os_error() == Some(11) => false,
                    Err(e) => {
                        println!("[ReFramed] read() failed, exiting listen thread: {}", e);
                        break;
                    }
                };

                // Forward data from the "send" fifo to the socket
                let data_was_sent = match rx.try_recv() {
                    Ok(data) => match stream.write(&data) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("[ReFramed] write() failed, exiting client thread: {}", e);
                            break;
                        }
                    },
                    Err(mpsc::TryRecvError::Disconnected) => {
                        println!("[ReFramed] Write end of send channel was closed, exiting client thread");
                        break;
                    },
                    Err(mpsc::TryRecvError::Empty) => false
                };

                // Since we're polling, we don't want to exchaust the CPU
                if !data_was_sent && !data_was_received {
                    thread::sleep(Duration::from_millis(50));
                }
            }
        });
    }


    pub fn listen_for_incoming_connections(&self) {
        let listener = match TcpListener::bind("0.0.0.0:42069") {
            Ok(listener) => listener,
            Err(e) => { println!("[ReFramed] Failed to bind socket: {}", e); return; }
        };

        match listener.local_addr() {
            Ok(addr) => println!("[ReFramed] Started server on {}", addr),
            Err(_) => println!("[ReFramed] Started server")
        };

        for stream in listener.incoming() {
            match stream {
                Ok(client) => self.accept_client(client),
                Err(e) => {
                    println!("[ReFramed] Accept failed: {}", e);
                    break;
                }
            }
        }
    }

    pub fn broadcast(&self, data: &[u8]) {
        self.clients.lock().unwrap().retain(|ref tx| {
            match tx.send(data.to_vec()) {
                Ok(_) => true,
                Err(e) => {
                    println!("[ReFramed] send() failed, removing client: {}", e);
                    false
                }
            }
        });
    }
}

