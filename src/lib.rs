#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate lazy_static;

mod server;
mod game_info;
mod protocol;
mod constants;

use skyline;
use acmd;
use smash::app::{utility, sv_system};
use smash::lib::lua_const;
use smash::app;
use smash::app::lua_bind;
use smash::lua2cpp::L2CFighterCommon;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

extern "C" {
    #[link_name="\u{1}_ZN3app14sv_information27get_remaining_time_as_frameEv"]
    pub fn get_remaining_time_as_frame() -> u32;

    #[link_name="\u{1}_ZN3app14sv_information8stage_idEv"]
    pub fn get_stage_id() -> i32;
}

lazy_static!{
    static ref SERVER: server::Server = server::Server::new();
    static ref GAME_INFO: Mutex<game_info::GameInfo> = Mutex::new(game_info::GameInfo::new());
}
static mut FIGHTER_MANAGER_ADDR: usize = 0;

pub fn once_per_fighter_frame(fighter : &mut L2CFighterCommon) {
    let lua_state = fighter.lua_state_agent;
    let module_accessor = unsafe { sv_system::battle_object_module_accessor(lua_state) };
    let fighter_manager = unsafe { *(FIGHTER_MANAGER_ADDR as *mut *mut app::FighterManager) };
    let fighter_entry_id = unsafe {
        lua_bind::WorkModule::get_int(module_accessor, *lua_const::FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as i32
    };

    // For now we only care about 1v1
    let num_fighters = unsafe { lua_bind::FighterManager::entry_count(fighter_manager) };
    if num_fighters != 2 {
        return;
    }

    let fighter_kind = unsafe { utility::get_kind(module_accessor) };

    let mut game_info = GAME_INFO.lock().unwrap();

    // Determine if a match started/ended
    if unsafe { lua_bind::FighterManager::is_result_mode(fighter_manager) } {
        if game_info.match_is_running() {
            game_info.set_match_end();
            protocol::send_match_end(&SERVER);
        }
    } else {
        if !game_info.match_is_running() {
            game_info.set_player_info(fighter_entry_id, &format!("Player {}", fighter_entry_id + 1), fighter_kind);
            game_info.set_stage(unsafe { get_stage_id() });
            if game_info.have_enough_info_to_start_match() {
                game_info.set_match_start();
                protocol::send_match_start(&SERVER, &game_info);
            }
        }
    }

    if !game_info.match_is_running() {
        return;
    }

    let fighter_information = unsafe {
        lua_bind::FighterManager::get_fighter_information(fighter_manager, app::FighterEntryID(fighter_entry_id)) as *mut app::FighterInformation
    };

    // TODO
    // Figure out when BoX sets start and end
    // Iframes
    // XY position

    let stock_count = unsafe { lua_bind::FighterInformation::stock_count(fighter_information) as u8 };
    let frames_left = unsafe { get_remaining_time_as_frame() };
    let fighter_status_kind = unsafe { lua_bind::StatusModule::status_kind(module_accessor) };
    let fighter_motion_kind = unsafe{ lua_bind::MotionModule::motion_kind(module_accessor) };
    let fighter_damage = unsafe { lua_bind::DamageModule::damage(module_accessor, 0) };
    let fighter_shield_size = unsafe { lua_bind::WorkModule::get_float(module_accessor, *lua_const::FIGHTER_INSTANCE_WORK_ID_FLOAT_GUARD_SHIELD) };
    let attack_connected = unsafe { lua_bind::AttackModule::is_infliction_status(module_accessor, *lua_const::COLLISION_KIND_MASK_HIT) };
    let hitstun_left = unsafe { lua_bind::WorkModule::get_float(module_accessor, *lua_const::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME) };
    let pos_x = unsafe { lua_bind::PostureModule::pos_x(module_accessor) };
    let pos_y = unsafe { lua_bind::PostureModule::pos_y(module_accessor) };
    let facing = unsafe { lua_bind::PostureModule::lr(module_accessor) };
    let iframe_status = unsafe { lua_bind::HitModule::get_total_status(module_accessor, 0) };

    protocol::send_fighter_info(&SERVER,
        frames_left,
        fighter_entry_id,
        pos_x,
        pos_y,
        facing,
        fighter_damage,
        hitstun_left,
        fighter_shield_size,
        fighter_status_kind,
        fighter_motion_kind,
        iframe_status,
        stock_count,
        attack_connected
    );
}

#[skyline::main(name = "uhrecorder")]
pub fn main() {
    unsafe {
        skyline::nn::ro::LookupSymbol(
            &mut FIGHTER_MANAGER_ADDR,
            "_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\u{0}".as_bytes().as_ptr(),
        );
    }
    acmd::add_custom_hooks!(once_per_fighter_frame);

    std::thread::spawn(move || {
        loop {
            SERVER.listen_for_incoming_connections();
            thread::sleep(Duration::from_secs(10))
        }
    });
}

