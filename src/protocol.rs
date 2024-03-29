use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::game_info::GameInfo;
use crate::training_info::TrainingInfo;
use crate::server::Server;
use crate::constants;
use crc::{Crc, CRC_32_CKSUM};
use skyline::libc;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageType {
    ProtocolVersion,

    MappingInfoChecksum,
    MappingInfoRequest,
    MappingInfoFighterKinds,
    MappingInfoFighterStatusKinds,
    MappingInfoStageKinds,
    MappingInfoHitStatusKinds,
    MappingInfoRequestComplete,


    MatchStart,
    MatchResume,
    MatchEnd,
    TrainingStart,
    TrainingResume,
    TrainingReset,
    TrainingEnd,

    FighterState,
}

fn send_bytes(socket: libc::c_int, bytes: &[u8]) -> Result<(), i64> {
    unsafe {
        let result = libc::send(socket, bytes.as_ptr() as *const _, bytes.len(), 0);
        if result < 0 {
            Err(*libc::errno_loc())
        } else {
            Ok(())
        }
    }
}

pub fn send_protocol_version(socket: libc::c_int) -> Result<(), i64> {
    let major = 0x01;
    let minor = 0x01;
    println!("[ReFramed] Sending protocol version {}.{}", major, minor);
    send_bytes(socket, &[MessageType::ProtocolVersion.into(), major, minor])?;
    Ok(())
}

fn calc_mapping_info_checksum() -> u32 {
    let crc = Crc::<u32>::new(&CRC_32_CKSUM);
    let mut digest = crc.digest();

    for (kind, name) in constants::FIGHTER_KINDS.iter() {
        digest.update(&(kind.as_lua_int().get_int() as u8).to_be_bytes());
        digest.update(name.as_bytes());
    }

    for (kind, name) in constants::STAGE_KINDS.iter() {
        digest.update(&(*kind as u16).to_be_bytes());
        digest.update(name.as_bytes());
    }

    for (status, fighter, name) in constants::FIGHTER_STATUS_KINDS.iter() {
        digest.update(&(fighter.as_lua_int().get_int() as u8).to_be_bytes());
        digest.update(&(status.as_lua_int().get_int() as u16).to_be_bytes());
        digest.update(name.as_bytes());
    }

    for (kind, name) in constants::HIT_STATUS_KINDS.iter() {
        digest.update(&(kind.as_lua_int().get_int() as u8).to_be_bytes());
        digest.update(name.as_bytes());
    }

    digest.finalize()
}

pub fn send_mapping_info_checksum(socket: libc::c_int) -> Result<(), i64> {
    let [h0, h1, h2, h3] = calc_mapping_info_checksum().to_be_bytes();
    println!("[ReFramed] Sending mapping info checksum");
    send_bytes(socket, &[MessageType::MappingInfoChecksum.into(), h0, h1, h2, h3])?;
    Ok(())
}

pub fn send_mapping_info(socket: libc::c_int) -> Result<(), i64> {
    println!("[ReFramed] Sending mapping info");

    let [h0, h1, h2, h3] = calc_mapping_info_checksum().to_be_bytes();
    send_bytes(socket, &[MessageType::MappingInfoRequest.into(), h0, h1, h2, h3])?;

    send_fighter_kind_constants(socket)?;
    send_fighter_status_kind_constants(socket)?;
    send_stage_constants(socket)?;
    send_hit_status_constants(socket)?;
    send_bytes(socket, &[MessageType::MappingInfoRequestComplete.into()])?;
    Ok(())
}

fn match_start_payload(info: &GameInfo) -> Vec<u8> {
    let stage_id = info.get_stage() as u16;
    let stage_id_u = ((stage_id >> 8) & 0xFF) as u8;
    let stage_id_l = ((stage_id >> 0) & 0xFF) as u8;
    let p1name_bytes = info.p1_name().as_bytes();
    let p2name_bytes = info.p2_name().as_bytes();

    let mut data = vec![
        stage_id_u, stage_id_l,
        2,  // 2 players, hardcoded for now
        info.p1_entry_id() as u8,
        info.p1_fighter_kind() as u8,
        info.p1_fighter_skin() as u8,
        info.p2_entry_id() as u8,
        info.p2_fighter_kind() as u8,
        info.p2_fighter_skin() as u8
    ];
    data.push(p1name_bytes.len() as u8);
    data.extend_from_slice(p1name_bytes);
    data.push(p2name_bytes.len() as u8);
    data.extend_from_slice(p2name_bytes);

    data
}

pub fn broadcast_match_start(server: &Server, info: &GameInfo) {
    let mut data = match_start_payload(&info);
    data.insert(0, MessageType::MatchStart.into());
    println!("[ReFramed] Match start: stage: {}, p1: {} ({}), p2: {} ({})",
        info.get_stage(),
        info.p1_name(), info.p1_fighter_kind(),
        info.p2_name(), info.p2_fighter_kind()
    );

    server.broadcast(&data);
}

pub fn send_match_resume(socket: libc::c_int) -> Result<(), i64> {
    let game_info = GameInfo::get().lock().unwrap();
    if game_info.match_is_running() {
        let mut data = match_start_payload(&game_info);
        data.insert(0, MessageType::MatchResume.into());

        println!("[ReFramed] Match resume: stage: {}, p1: {} ({}), p2: {} ({})",
            game_info.get_stage(),
            game_info.p1_name(), game_info.p1_fighter_kind(),
            game_info.p2_name(), game_info.p2_fighter_kind()
        );
        send_bytes(socket, &data)?;
    }
    Ok(())
}

pub fn broadcast_match_end(server: &Server) {
    println!("[ReFramed] Match end");
    server.broadcast(&[MessageType::MatchEnd.into()]);
}

fn training_start_payload(info: &TrainingInfo) -> [u8; 4] {
    let stage_id = info.get_stage();
    let stage_id_u = ((stage_id >> 8) & 0xFF) as u8;
    let stage_id_l = ((stage_id >> 0) & 0xFF) as u8;
    [
        stage_id_u, stage_id_l,
        info.p1_fighter_kind() as u8,
        info.cpu_fighter_kind() as u8
    ]
}

pub fn broadcast_training_start(server: &Server, info: &TrainingInfo) {
    println!("[ReFramed] Training start: stage: {}, p1: {}, cpu: {}",
        info.get_stage(),
        info.p1_fighter_kind(),
        info.cpu_fighter_kind()
    );

    let mut data: Vec<u8> = Vec::new();
    data.push(MessageType::TrainingStart.into());
    data.extend_from_slice(&training_start_payload(&info));
    server.broadcast(&data);
}

pub fn send_training_resume(socket: libc::c_int) -> Result<(), i64> {
    let training_info = TrainingInfo::get().lock().unwrap();
    if training_info.is_running() {
        let mut data: Vec<u8> = Vec::new();
        data.push(MessageType::TrainingResume.into());
        data.extend_from_slice(&training_start_payload(&training_info));

        println!("[ReFramed] Training resume: stage: {}, p1: {}, cpu: {}",
            training_info.get_stage(),
            training_info.p1_fighter_kind(),
            training_info.cpu_fighter_kind()
        );
        send_bytes(socket, &data)?;
    }
    Ok(())
}

pub fn broadcast_training_end(server: &Server) {
    println!("[ReFramed] Training end");
    server.broadcast(&[MessageType::TrainingEnd.into()]);
}

pub fn broadcast_fighter_info(
    server: &Server,
    frame: u32,
    entry_id: i32,
    pos_x: f32,
    pos_y: f32,
    facing: f32,
    damage: f32,
    _hitlag_left: u64,  // This value doesn't work for all attacks, so better not send it until we find a solution
    hitstun_left: f32,
    shield_size: f32,
    status_kind: i32,
    motion_kind: u64,
    hit_status: u64,
    stock_count: u8,
    attack_connected: bool,
    opponent_in_hitlag: bool
) {
    // We don't really need to know damage beyond 0.02% accuracy and the upper
    // limit is 999.99%, so multiplying it by 50 lets us store it in one u16
    let damage_int = (damage*50.0) as u16;
    let [damage0, damage1] = damage_int.to_be_bytes();

    // Shield sizes seem to be around 50ish -> 10000 max leaves some room
    let shield_int = (shield_size*200.0) as u16;
    let [shield0, shield1] = shield_int.to_be_bytes();

    // Can't think of any move with hitstun over 1 second (60)
    // 60*100 = 60000
    let hitstun_int = (hitstun_left*100.0) as u16;
    let [hitstun0, hitstun1] = hitstun_int.to_be_bytes();

    // Highest status value seems to be 872 (I'm looking at you kirby)
    let [status0, status1] = (status_kind as u16).to_be_bytes();

    // Motion kinds are hash40 values which use 40 bits (5 bytes)
    let [_, _, _, motion0, motion1, motion2, motion3, motion4] = motion_kind.to_be_bytes();

    // Booleans can be combined into a single u8
    let flags = 
        ((attack_connected as u8) << 0)
      | ((if facing > 0.0 {1} else {0}) << 1)
      | ((opponent_in_hitlag as u8) << 2);

    // Other 
    let [frame0, frame1, frame2, frame3] = frame.to_be_bytes();
    let [posx0, posx1, posx2, posx3] = pos_x.to_be_bytes();
    let [posy0, posy1, posy2, posy3] = pos_y.to_be_bytes();

    server.broadcast(&[
        MessageType::FighterState.into(),
        frame0, frame1, frame2, frame3,
        entry_id as u8,
        posx0, posx1, posx2, posx3,
        posy0, posy1, posy2, posy3,
        damage0, damage1,
        hitstun0, hitstun1,
        shield0, shield1,
        status0, status1,
        motion0, motion1, motion2, motion3, motion4,
        hit_status as u8,
        stock_count,
        flags
    ]);
}

fn send_fighter_kind_constants(socket: libc::c_int) -> Result<(), i64> {
    let mut buf = vec![];
    for (kind, name) in constants::FIGHTER_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let data = &[MessageType::MappingInfoFighterKinds.into(), kind.as_lua_int().get_int() as u8, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        buf.extend_from_slice(data);
    }
    send_bytes(socket, &buf)?;
    Ok(())
}

fn send_stage_constants(socket: libc::c_int) -> Result<(), i64> {
    let mut buf = vec![];
    for (kind, name) in constants::STAGE_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let [kind0, kind1] = (*kind as u16).to_be_bytes();
        let data = &[MessageType::MappingInfoStageKinds.into(), kind0, kind1, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        buf.extend_from_slice(data);
    }
    send_bytes(socket, &buf)?;
    Ok(())
}

fn send_fighter_status_kind_constants(socket: libc::c_int) -> Result<(), i64> {
    for (status, fighter, name) in constants::FIGHTER_STATUS_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let [status0, status1] = (status.as_lua_int().get_int() as u16).to_be_bytes();
        let data = &[MessageType::MappingInfoFighterStatusKinds.into(), fighter.as_lua_int().get_int() as u8, status0, status1, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        send_bytes(socket, &data)?;

        // Something horrible happens if we don't do this
        std::thread::sleep(std::time::Duration::from_millis(3));
    }
    Ok(())
}

fn send_hit_status_constants(socket: libc::c_int) -> Result<(), i64> {
    let mut buf = vec![];
    for (kind, name) in constants::HIT_STATUS_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let data = &[MessageType::MappingInfoHitStatusKinds.into(), kind.as_lua_int().get_int() as u8, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        buf.extend_from_slice(data);
    }
    send_bytes(socket, &buf)?;
    Ok(())
}

