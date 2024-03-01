#![feature(proc_macro_hygiene)]
#[macro_use]

mod constants;
mod player_tags;
mod game_info;
mod protocol;
//mod replay;
mod server;
mod training_info;

use training_info::TrainingInfo;
use game_info::GameInfo;
//use replay::ReplayManager;
use lazy_static::lazy_static;
use skyline;
use acmd;
use smash::app::{utility, sv_system, smashball};
use smash::lib::{lua_const, L2CValue};
use smash::app;
use smash::app::lua_bind;
use smash::lua2cpp::{L2CFighterCommon, L2CFighterBase, L2CFighterBase_global_reset};
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
}
static mut FIGHTER_MANAGER_ADDR: usize = 0;

/*
 * Training start
 * [ReFramed] global_init() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * ...
 *
 * Switch to pyra
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] global_reset() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * ...
 *
 * Reset training mode
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * ...
 *
 * Exit training mode
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: true
 * (no more callbacks)
 *
 * Start 1v1
 * [ReFramed] global_init() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * [ReFramed] global_init() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: false, is_training_mode: false 
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * ...
 *
 * Switch to pyra
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 *
 * End 1v1
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: false, is_training_mode: false
 * ...
 * [ReFramed] global_init() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * ...
 * 
 * Quit 1v1
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: true, is_result_mode: false, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] global_reset() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * [ReFramed] frame() -- is_ready_go: false, is_result_mode: true, is_training_mode: false
 * ...
 */

#[skyline::hook(replace = L2CFighterBase_global_reset)]
pub fn handle_fighter_global_reset(fighter: &mut L2CFighterBase) -> L2CValue {
    let fighter_manager = unsafe { *(FIGHTER_MANAGER_ADDR as *mut *mut app::FighterManager) };
    let is_ready_go = unsafe { lua_bind::FighterManager::is_ready_go(fighter_manager) };
    let is_result_mode = unsafe { lua_bind::FighterManager::is_result_mode(fighter_manager) };
    let is_training_mode = unsafe { smashball::is_training_mode() };

    if is_training_mode {
        let mut training_info = TrainingInfo::get().lock().unwrap();
        if !training_info.is_running() {
            training_info.set_start_pending();
        }
    }

    if !is_training_mode && !is_ready_go && is_result_mode {
        let mut game_info = GameInfo::get().lock().unwrap();
        if game_info.match_is_running() {
            game_info.set_match_end();
            protocol::broadcast_match_end(&SERVER);
        }
    }

    original!()(fighter)
}

pub fn once_per_frame_per_fighter(fighter : &mut L2CFighterCommon) {
    let lua_state = fighter.lua_state_agent;
    let module_accessor = unsafe { sv_system::battle_object_module_accessor(lua_state) };
    let fighter_manager = unsafe { *(FIGHTER_MANAGER_ADDR as *mut *mut app::FighterManager) };
    let fighter_entry_id = unsafe { lua_bind::WorkModule::get_int(module_accessor, *lua_const::FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as i32 };
    let fighter_skin = unsafe { lua_bind::WorkModule::get_int(module_accessor, *lua_const::FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as i32 };
    let fighter_kind = unsafe { utility::get_kind(module_accessor) };
    let is_ready_go = unsafe { lua_bind::FighterManager::is_ready_go(fighter_manager) };
    let is_training_mode = unsafe { smashball::is_training_mode() };

    if is_training_mode {
        let mut training_info = TrainingInfo::get().lock().unwrap();

        // Start notification logic. Have to collect info over multiple
        // callbacks to this function before being able to send
        // the start event, but the actual detection of the start
        // event happens in the global_reset() hook.
        if is_ready_go && training_info.is_start_pending() {
            if fighter_entry_id == 0 {
                training_info.set_player_info(fighter_kind);
            }
            if fighter_entry_id == 1 {
                training_info.set_cpu_info(fighter_kind);
            }
            training_info.set_stage(unsafe { get_stage_id() });

            if training_info.have_enough_info_to_start() {
                training_info.start();
                protocol::broadcast_training_start(&SERVER, &training_info);
            }
        }

        // Stop notification logic
        if !is_ready_go && training_info.is_running() {
            training_info.stop();
            protocol::broadcast_training_end(&SERVER);
        }

        // Don't send player states if training mode hasn't started
        if !training_info.is_running() {
            return;
        }
    } else {
        // For now we only care about 1v1
        let num_fighters = unsafe { lua_bind::FighterManager::entry_count(fighter_manager) };
        if num_fighters != 2 {
            return;
        }

        let mut game_info = GameInfo::get().lock().unwrap();

        // Start notification logic. Have to collect info over multiple
        // callbacks to this function before being able to send the
        // start event.
        if is_ready_go && !game_info.match_is_running() {
            let player_tag = player_tags::get_name_for_slot(fighter_entry_id);
            let player_tag = if player_tag.is_empty() {
                format!("Player {}", fighter_entry_id + 1)
            } else {
                player_tag
            };

            game_info.set_player_info(
                    fighter_entry_id,
                    &player_tag,
                    fighter_kind,
                    fighter_skin);
            game_info.set_stage(unsafe { get_stage_id() });

            if game_info.have_enough_info_to_start_match() {
                game_info.set_match_start();
                protocol::broadcast_match_start(&SERVER, &game_info);
            }
        }

        // Don't send player states if match hasn't started
        if !game_info.match_is_running() {
            return;
        }
    }

    let fighter_information = unsafe {
        lua_bind::FighterManager::get_fighter_information(fighter_manager, app::FighterEntryID(fighter_entry_id)) as *mut app::FighterInformation
    };

    // TODO
    // Figure out when BoX sets start and end
    // Iframes

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

    // Hitlag frames left on the opponent. Weird, seems to only work for
    // "normal" attacks, not electric (such as pika dair, nair, tjolt). Same
    // goes for the is_stop flag.
    let hitlag_left = unsafe { lua_bind::StopModule::get_hit_stop_real_frame(module_accessor) };
    //let is_stop = unsafe { lua_bind::StopModule::is_stop(module_accessor) };

    // This is true if the opponent is in hitlag
    let opponent_in_hitlag = unsafe { lua_bind::FighterStopModuleImpl::is_damage_stop(module_accessor) };

    //let button = unsafe { lua_bind::ControlModule::get_button(module_accessor) };

    //println!("hitlag_left: {}, opponent_in_hitlag: {}, button: {:#04x}", hitlag_left, opponent_in_hitlag, button as u32);

    protocol::broadcast_fighter_info(&SERVER,
        frames_left,
        fighter_entry_id,
        pos_x,
        pos_y,
        facing,
        fighter_damage,
        hitlag_left,
        hitstun_left,
        fighter_shield_size,
        fighter_status_kind,
        fighter_motion_kind,
        iframe_status,
        stock_count,
        attack_connected,
        opponent_in_hitlag,
    );
}

fn nro_main(nro: &skyline::nro::NroInfo<'_>) {
    match nro.name {
        "common" => {
            skyline::install_hooks!(
                handle_fighter_global_reset,
            );
        },
        _ => (),
    }
}

#[skyline::main(name = "ReFramed")]
pub fn main() {
    skyline::nro::add_hook(nro_main).unwrap();
    unsafe {
        skyline::nn::ro::LookupSymbol(
            &mut FIGHTER_MANAGER_ADDR,
            "_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\u{0}".as_bytes().as_ptr(),
        );
    }
    acmd::add_custom_hooks!(once_per_frame_per_fighter);

    std::thread::spawn(move || {
        loop {
            SERVER.listen_for_incoming_connections();
            thread::sleep(Duration::from_secs(10))
        }
    });
}

