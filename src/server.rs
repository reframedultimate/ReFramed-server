use std::vec::Vec;
use std::thread;
use std::sync;
use std::mem::size_of_val;
use skyline::libc;

use crate::protocol;

pub struct Client {
    socket: libc::c_int,
    allow_broadcasts: bool
}

impl Client {
    pub fn new(socket: libc::c_int) -> Self {
        Self {
            socket: socket,
            allow_broadcasts: false,
        }
    }

    pub fn socket(&self) -> libc::c_int {
        self.socket
    }

    pub fn set_allow_broadcasts(&mut self) {
        self.allow_broadcasts = true;
    }

    pub fn allow_broadcasts(&self) -> bool {
        self.allow_broadcasts
    }
}

pub struct Server {
    clients: sync::Mutex<Vec<sync::Mutex<Client>>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: sync::Mutex::new(Vec::new())
        }
    }

    fn start_client_read_thread(&self, socket: libc::c_int /*, client: &sync::Mutex<Client>*/) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                let mut buf: [u8; 1] = [0; 1];
                unsafe {
                    if libc::recv(socket, &mut buf as *mut _ as *mut libc::c_void, 1, 0) < 1 {
                        break;
                    }
                }

                let send_result = match protocol::MessageType::try_from(buf[0]) {
                    Ok(protocol::MessageType::ProtocolVersion) => protocol::send_protocol_version(socket),
                    Ok(protocol::MessageType::MappingInfoChecksum) => protocol::send_mapping_info_checksum(socket),
                    Ok(protocol::MessageType::MappingInfoRequest) => protocol::send_mapping_info(socket),
                    Ok(protocol::MessageType::MappingInfoFighterKinds) => { Ok(()) },
                    Ok(protocol::MessageType::MappingInfoFighterStatusKinds) => { Ok(()) },
                    Ok(protocol::MessageType::MappingInfoStageKinds) => { Ok(()) },
                    Ok(protocol::MessageType::MappingInfoHitStatusKinds) => { Ok(()) },
                    Ok(protocol::MessageType::MappingInfoRequestComplete) => { Ok(()) },
                    Ok(protocol::MessageType::MatchStart) => { Ok(()) },
                    Ok(protocol::MessageType::MatchResume) => {
                        //client.lock().unwrap().set_allow_broadcasts();
                        protocol::send_match_resume(socket)
                    },
                    Ok(protocol::MessageType::MatchEnd) => { Ok(()) },
                    Ok(protocol::MessageType::TrainingStart) => { Ok(()) },
                    Ok(protocol::MessageType::TrainingResume) => {
                        //client.lock().unwrap().set_allow_broadcasts();
                        protocol::send_training_resume(socket)
                    },
                    Ok(protocol::MessageType::TrainingEnd) => { Ok(()) },
                    Ok(protocol::MessageType::TrainingReset) => { Ok(()) },
                    Ok(protocol::MessageType::FighterState) => { Ok(()) },
                    Err(_) => {
                        println!("[ReFramed] Received unknown message type {} from client", buf[0]);
                        Ok(())
                    }
                };

                match send_result {
                    Ok(_) => {},
                    Err(errno) => {
                        println!("[ReFramed] Failed to write to client socket: {}", errno);
                        break;
                    }
                }
            }

            println!("[ReFramed] Closing client socket");
            unsafe {
                libc::close(socket);
            }
        })
    }

    pub fn listen_for_incoming_connections(&self) {
        let server_addr: libc::sockaddr_in = libc::sockaddr_in {
            sin_family: libc::AF_INET as _,
            sin_port: 42069_u16.to_be(),
            sin_len: 4,
            sin_addr: libc::in_addr {
                s_addr: libc::INADDR_ANY as _,
            },
            sin_zero: [0; 8],
        };

        // Create socket
        let socket = unsafe {
            libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0)
        };
        if (socket as u32 & 0x80000000) != 0 {
            println!("[ReFramed] Failed to create socket: {}", unsafe { *libc::errno_loc() });
            return;
        }

        // Enable keep-alive
        let flags: u32 = 1;
        unsafe {
            if libc::setsockopt(socket,
                    libc::SOL_SOCKET,
                    libc::SO_KEEPALIVE,
                    &flags as *const _ as *const libc::c_void,
                    size_of_val(&flags) as u32) < 0 {
                println!("[ReFramed] Failed to set socket options: {}", *libc::errno_loc());
                libc::close(socket);
                return;
            }
        }

        // Bind socket
        unsafe {
            if libc::bind(socket,
                    &server_addr as *const libc::sockaddr_in as *const libc::sockaddr,
                    size_of_val(&server_addr) as u32) < 0 {
                println!("[ReFramed] Failed to bind socket: {}", *libc::errno_loc());
                libc::close(socket);
                return;
            }
        }

        // Server loop
        println!("[ReFramed] Started server");
        loop {
            // Listen for incoming connection
            unsafe {
                if libc::listen(socket, 1) < 0 {
                    println!("[ReFramed] Failed to listen: {}", *libc::errno_loc());
                    break;
                }
            }

            // Accept incoming connection
            let mut addr_len: u32 = 0;
            let client_socket = unsafe {
                libc::accept(socket,
                        &server_addr as *const libc::sockaddr_in as *mut libc::sockaddr,
                        &mut addr_len)
            };
            if (client_socket as u32 & 0x80000000) != 0 {
                println!("[ReFramed] Failed to accept client connection: {}", unsafe { *libc::errno_loc() });
                break;
            }

            let client = sync::Mutex::new(Client::new(client_socket));
            let mut clients = self.clients.lock().unwrap();
            clients.push(client);
            self.start_client_read_thread(client_socket);
        }
        println!("[ReFramed] Stopping server...");
        for client in self.clients.lock().unwrap().iter() {
            unsafe {
                libc::shutdown(client.lock().unwrap().socket(), libc::SHUT_RDWR);
            }
        }

        unsafe {
            libc::close(socket);
        }
    }

    pub fn broadcast(&self, data: &[u8]) {
        self.clients.lock().unwrap().retain(|client| {
            let socket = client.lock().unwrap().socket();
            let result = unsafe {
                libc::send(socket, data.as_ptr() as *const _, data.len(), 0)
            };
            if result < 0 {
                println!("[ReFramed] send() failed with {}, removing client", unsafe { *libc::errno_loc() });
                unsafe {
                    libc::shutdown(socket, libc::SHUT_RDWR);
                }
                false
            } else {
                true
            }
        });
    }
}

