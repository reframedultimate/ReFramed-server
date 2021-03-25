use crate::protocol::Protocol;

use std::sync::Mutex;
use std::vec::Vec;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error, ErrorKind};
use std::convert::From;

pub enum ServerError {
    AddressInUse,
    Other
}

pub struct Server {
    clients: Mutex<Vec<TcpStream>>
}

impl From<Error> for ServerError {
    fn from(err: Error) -> ServerError {
        match err.kind() {
            ErrorKind::AddrInUse => ServerError::AddressInUse,
            _ => ServerError::Other
        }
    }
}

impl Protocol for Server {}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: Mutex::new(Vec::new())
        }
    }

    pub fn listen_for_incoming_connections(&self) -> Result<(), ServerError> {
        let listener = TcpListener::bind("0.0.0.0:42069")?;

        println!("[uhrecorder] Started server on {}", listener.local_addr()?);

        for stream in listener.incoming() {
            self.poll();
            match stream {
                Ok(mut stream) => {
                    println!("[uhrecorder] Accepted client connection {}", stream.local_addr()?);
                    stream.set_nonblocking(true)?;
                    Server::send_constants(&mut stream)?;
                    self.clients.lock().unwrap().push(stream);
                },
                Err(e) => println!("[uhrecorder] Failed to accept client connection: {}", e)
            };
        }

        Ok(())
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

