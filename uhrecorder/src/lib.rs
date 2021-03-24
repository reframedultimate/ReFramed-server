#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use std::io::Write;
use skyline;
use acmd;
use smash::app::{utility, sv_system};
use smash::lib::lua_const;
use smash::lib::L2CValue;
use smash::lua2cpp::{L2CFighterCommon, L2CFighterCommon_status_pre_Rebirth, L2CFighterCommon_status_pre_Entry, L2CFighterCommon_sub_damage_uniq_process_init};

extern "C" {
    #[link_name="\u{1}_ZN3app14sv_information27get_remaining_time_as_frameEv"]
    pub fn get_remaining_time_as_frame() -> u32;

    #[link_name="\u{1}_ZN3app14sv_information8stage_idEv"]
    pub fn get_stage_id() -> i32;
}

enum ServerError {
    AddressInUse,
    Other
}

#[derive(Default)]
struct Server {
    clients: std::sync::Mutex<std::vec::Vec<std::net::TcpStream>>
}

impl Server {
    fn new() -> Server {
        Server {
            clients: std::sync::Mutex::new(std::vec::Vec::new())
        }
    }

    fn listen_for_incoming_connections(&self) -> Result<(), ServerError> {
        let listener = match std::net::TcpListener::bind("0.0.0.0:42069") {
            Ok(listener) => Ok(listener),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AddrInUse => Err(ServerError::AddressInUse),
                _ => Err(ServerError::Other)
            }
        }?;

        match listener.local_addr() {
            Ok(addr) => println!("[uhrecorder] Started server on {}", addr),
            _ => ()
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    match stream.local_addr() {
                        Ok(addr) => println!("[uhrecorder] Accepted client connection {}", addr),
                        Err(e) => println!("[uhrecorder] Accepted client connection (failed to get local addr but accepting anyway: {})", e)
                    }
                    self.clients.lock().unwrap().push(stream);
                },
                Err(e) => {
                    println!("[uhrecorder] Failed to accept client connection: {}", e);
                }
            }
        }

        Ok(())
    }

    fn broadcast(&self, msg: &std::string::String) {
        for client in self.clients.lock().unwrap().iter_mut() {
            if let Err(e) = client.write(msg.as_bytes()) {
                break;  // TODO remove client
            }
        }
    }
}

lazy_static! { static ref SERVER : Server = Server::new(); }

pub fn once_per_fighter_frame(fighter : &mut L2CFighterCommon) {
    let lua_state = fighter.lua_state_agent;
    let fighter_kind = unsafe {
        let module_accessor = sv_system::battle_object_module_accessor(lua_state);
        utility::get_kind(module_accessor)
    };
    let frames_left = unsafe { get_remaining_time_as_frame() };
    let stage_id = unsafe { get_stage_id() };

    SERVER.broadcast(&format!(
        "f{}: kind {}, stage: {}\n", frames_left, fighter_kind, stage_id
    ));

    if fighter_kind == *lua_const::FIGHTER_KIND_FOX {
    }
}

#[skyline::hook(replace = L2CFighterCommon_status_pre_Entry)]
pub fn handle_pre_entry(fighter: &mut L2CFighterCommon) -> L2CValue {
    let lua_state = fighter.lua_state_agent;
    let fighter_kind = unsafe {
        let module_accessor = sv_system::battle_object_module_accessor(lua_state);
        utility::get_kind(module_accessor)
    };

    println!("pre_entry: {}", fighter_kind);
    original!()(fighter)
}

#[skyline::hook(replace = L2CFighterCommon_status_pre_Rebirth)]
pub fn handle_pre_rebirth(fighter: &mut L2CFighterCommon) -> L2CValue {
    let lua_state = fighter.lua_state_agent;
    let fighter_kind = unsafe {
        let module_accessor = sv_system::battle_object_module_accessor(lua_state);
        utility::get_kind(module_accessor)
    };

    println!("pre_rebirth: {}", fighter_kind);
    original!()(fighter)
}

#[skyline::hook(replace = L2CFighterCommon_sub_damage_uniq_process_init)]
pub fn handle_sub_damage_uniq_process_init(fighter: &mut L2CFighterCommon) -> L2CValue {
    let lua_state = fighter.lua_state_agent;
    let fighter_kind = unsafe {
        let module_accessor = sv_system::battle_object_module_accessor(lua_state);
        utility::get_kind(module_accessor)
    };

    println!("sub_damage: {}", fighter_kind);
    original!()(fighter)
}


fn nro_main(nro: &skyline::nro::NroInfo<'_>) {
    match nro.name {
        "common" => {
            skyline::install_hooks!(
                handle_pre_entry,
                handle_pre_rebirth,
                handle_sub_damage_uniq_process_init
            );
        }
        _ => (),
    }
}

#[skyline::main(name = "uhrecorder")]
pub fn main() {
    skyline::nro::add_hook(nro_main).unwrap();
    acmd::add_custom_hooks!(once_per_fighter_frame);

    std::thread::spawn(||{
        loop {
            match SERVER.listen_for_incoming_connections() {
                Ok(()) => break,
                Err(ServerError::AddressInUse) => break,
                _ => ()
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });
}

