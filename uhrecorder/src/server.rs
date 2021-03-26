use std::sync::Mutex;
use std::vec::Vec;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error};
use std::thread;
use std::time::Duration;

use crate::protocol;

pub struct Server {
    clients: Mutex<Vec<TcpStream>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: Mutex::new(Vec::new())
        }
    }

    pub fn run(&self) {
        loop {
            self.listen_for_incoming_connections();
            thread::sleep(Duration::from_secs(5));
        }
    }

    fn accept_client(&self, stream_result: Result<TcpStream, Error>) -> Result<(), Error> {
        let mut client = stream_result?;
        protocol::send_constants(&mut client)?;
        client.set_nonblocking(true)?;

        println!("[uhrecorder] Accepted client connection {}", client.local_addr()?);
        self.clients.lock().unwrap().push(client);
        Ok(())
    }

    fn listen_for_incoming_connections(&self) {
        // I have no idea if the thread even exits after a sleep cycle, but it can't hurt
        // to make sure any old clients are discarded
        self.clients.lock().unwrap().clear();

        let listener = match TcpListener::bind("0.0.0.0:42069") {
            Ok(listener) => listener,
            Err(e) => { println!("[uhrecorder] Failed to bind socket: {}", e); return; }
        };

        println!("[uhrecorder] Started server on {}", listener.local_addr().unwrap());

        for stream in listener.incoming() {
            self.poll();
            match self.accept_client(stream) {
                Err(e) => println!("[uhrecorder] Failed to accept client connection: {}", e),
                _ => ()
            };
        }

        self.clients.lock().unwrap().clear();
    }

    pub fn broadcast(&self, data: &[u8]) {
        self.poll();
        self.clients.lock().unwrap().retain(|ref mut client| {
            match client.write(data) {
                Ok(_) => true,
                Err(e) => {
                    println!("[uhrecorder] Closing connection to client: {}", e);
                    false
                }
            }
        });
    }

    fn poll(&self) {
        self.clients.lock().unwrap().retain(|ref mut client| {
            let mut buf = vec![];
            match client.read_to_end(&mut buf) {
                Ok(_) => {
                    // TODO handle incoming data
                    true
                },
                // ErrorKind::WouldBlock doesn't seem to handle EAGAIN(11)
                Err(e) => match e.raw_os_error() {
                    Some(11) => true,
                    _ => {
                        println!("[uhrecorder] Closing connection to client: {}", e);
                        false
                    }
                }
            }
        });
    }
}

