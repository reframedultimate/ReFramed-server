#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

mod server;
mod protocol;

use skyline;
use acmd;
use smash::app::{utility, sv_system};
use smash::lib::lua_const;
use smash::lib::L2CValue;
use smash::app::lua_bind::StatusModule;
use smash::lua2cpp::{L2CFighterCommon, L2CFighterCommon_status_pre_Rebirth, L2CFighterCommon_status_pre_Entry, L2CFighterCommon_status_pre_Win, L2CFighterCommon_status_pre_Dead, L2CFighterCommon_sub_damage_uniq_process_init};

extern "C" {
    #[link_name="\u{1}_ZN3app14sv_information27get_remaining_time_as_frameEv"]
    pub fn get_remaining_time_as_frame() -> u32;

    #[link_name="\u{1}_ZN3app14sv_information8stage_idEv"]
    pub fn get_stage_id() -> i32;
}

lazy_static! { static ref SERVER : server::Server = server::Server::new(); }

pub fn once_per_fighter_frame(fighter : &mut L2CFighterCommon) {
    let lua_state = fighter.lua_state_agent;
    let module_accessor = unsafe { sv_system::battle_object_module_accessor(lua_state) };
    let fighter_kind = unsafe { utility::get_kind(module_accessor) };
    let status_kind = unsafe { StatusModule::status_kind(module_accessor) };
    let frames_left = unsafe { get_remaining_time_as_frame() };
    let stage_id = unsafe { get_stage_id() };

    SERVER.broadcast(&format!(
        "f{}: stage: {}, fighter: {}, status: {}\n", frames_left, stage_id, fighter_kind, status_kind
    ).as_bytes());

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

#[skyline::hook(replace = L2CFighterCommon_status_pre_Win)]
pub fn handle_pre_win(fighter: &mut L2CFighterCommon) -> L2CValue {
    println!("pre_win");
    original!()(fighter)
}

#[skyline::hook(replace = L2CFighterCommon_status_pre_Dead)]
pub fn handle_pre_dead(fighter: &mut L2CFighterCommon) -> L2CValue {
    println!("pre_dead");
    original!()(fighter)
}

fn nro_main(nro: &skyline::nro::NroInfo<'_>) {
    match nro.name {
        "common" => {
            skyline::install_hooks!(
                handle_pre_entry,
                handle_pre_rebirth,
                handle_pre_win,
                handle_pre_dead,
                handle_sub_damage_uniq_process_init,
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
                Err(server::ServerError::AddressInUse) => break,
                _ => ()
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });
}

