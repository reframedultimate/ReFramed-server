use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::game_info::GameInfo;
use crate::training_info::TrainingInfo;
use crate::server::Server;
use crate::constants;
use std::net::TcpStream;
use std::io::Write;
use std::convert::TryFrom;
use crc::{Crc, CRC_32_CKSUM};

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum MessageType {
    ProtocolVersion,

    MappingInfoChecksum,
    MappingInfoRequest,
    MappingInfoFighterKinds,
    MappingInfoFighterStatusKinds,
    MappingInfoStageKinds,
    MappingInfoHitStatusKinds,
    MappingInfoRequestComplete,

    MatchStart,
    MatchEnd,
    TrainingStart,
    TrainingReset,
    TrainingEnd,

    FighterState,
}

pub fn recv_data(buf: &[u8; 32], len: usize, stream: &TcpStream) -> std::io::Result<()> {
    for i in 0..len {
        match MessageType::try_from(buf[i]) {
            Ok(MessageType::ProtocolVersion) => send_protocol_version(stream)?,
            Ok(MessageType::MappingInfoChecksum) => send_mapping_info_checksum(stream)?,
            Ok(MessageType::MappingInfoRequest) => send_mapping_info(stream)?,
            Ok(MessageType::MappingInfoFighterKinds) => {},
            Ok(MessageType::MappingInfoFighterStatusKinds) => {},
            Ok(MessageType::MappingInfoStageKinds) => {},
            Ok(MessageType::MappingInfoHitStatusKinds) => {},
            Ok(MessageType::MappingInfoRequestComplete) => {},
            Ok(MessageType::MatchStart) => {},
            Ok(MessageType::MatchEnd) => {},
            Ok(MessageType::TrainingStart) => {},
            Ok(MessageType::TrainingEnd) => {},
            Ok(MessageType::TrainingReset) => {},
            Ok(MessageType::FighterState) => {},
            Err(_) => {}
        }
    }
    Ok(())
}

fn send_protocol_version(mut stream: &TcpStream) -> std::io::Result<()> {
    let major = 0x01;
    let minor = 0x00;
    println!("[ReFramed] Sending protocol version {}.{}", major, minor);
    stream.write(&[MessageType::ProtocolVersion.into(), major, minor])?;
    Ok(())
}

fn send_mapping_info_checksum(mut stream: &TcpStream) -> std::io::Result<()> {
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

    let [h0, h1, h2, h3] = digest.finalize().to_be_bytes();
    stream.write(&[MessageType::MappingInfoChecksum.into(), h0, h1, h2, h3])?;
    println!("[ReFramed] Sending mapping info hash");
    Ok(())
}

pub fn send_mapping_info(mut stream: &TcpStream) -> std::io::Result<()> {
    stream.write(&[MessageType::MappingInfoRequest.into()])?;
    send_fighter_kind_constants(stream)?;
    send_fighter_status_kind_constants(stream)?;
    send_stage_constants(stream)?;
    send_hit_status_constants(stream)?;
    stream.write(&[MessageType::MappingInfoRequestComplete.into()])?;
    Ok(())
}

pub fn send_match_start(server: &Server, info: &GameInfo) {
    let stage_id = info.get_stage() as u16;
    let stage_id_u = ((stage_id >> 8) & 0xFF) as u8;
    let stage_id_l = ((stage_id >> 0) & 0xFF) as u8;
    let p1name_bytes = info.p1_name().as_bytes();
    let p2name_bytes = info.p2_name().as_bytes();

    let mut data = vec![
        MessageType::MatchStart.into(),
        stage_id_u, stage_id_l,
        2,  // 2 players, hardcoded for now
        info.p1_entry_id() as u8,
        info.p2_entry_id() as u8,
        info.p1_fighter_kind() as u8,
        info.p2_fighter_kind() as u8
    ];
    data.push(p1name_bytes.len() as u8);
    data.extend_from_slice(p1name_bytes);
    data.push(p2name_bytes.len() as u8);
    data.extend_from_slice(p2name_bytes);

    println!("[ReFramed] Match start: stage: {}, p1: {} ({}), p2: {} ({})",
        stage_id,
        info.p1_name(), info.p1_fighter_kind(),
        info.p2_name(), info.p2_fighter_kind()
    );

    server.broadcast(&data);
}

pub fn send_match_end(server: &Server) {
    println!("[ReFramed] Match end");
    server.broadcast(&[MessageType::MatchEnd.into()]);
}

pub fn send_training_start(server: &Server, info: &TrainingInfo) {
    let stage_id = info.get_stage();
    let stage_id_u = ((stage_id >> 8) & 0xFF) as u8;
    let stage_id_l = ((stage_id >> 0) & 0xFF) as u8;
    let data = &[
        MessageType::TrainingStart.into(),
        stage_id_u, stage_id_l,
        info.p1_fighter_kind() as u8,
        info.cpu_fighter_kind() as u8
    ];

    println!("[ReFramed] Training Start: stage: {}, p1: {}, cpu: {}",
        stage_id,
        info.p1_fighter_kind(),
        info.cpu_fighter_kind()
    );

    server.broadcast(data);
}

pub fn send_training_end(server: &Server) {
    println!("[ReFramed] Training end");
    server.broadcast(&[MessageType::TrainingEnd.into()]);
}

pub fn send_fighter_info(
    server: &Server,
    frame: u32,
    entry_id: i32,
    pos_x: f32,
    pos_y: f32,
    facing: f32,
    damage: f32,
    hitstun_left: f32,
    shield_size: f32,
    status_kind: i32,
    motion_kind: u64,
    hit_status_status: u64,
    stock_count: u8,
    attack_connected: bool
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
      | ((if facing > 0.0 {1} else {0}) << 1);

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
        hit_status_status as u8,
        stock_count,
        flags
    ]);
}

fn send_fighter_kind_constants(mut stream: &TcpStream) -> std::io::Result<()> {
    let mut buf = vec![];
    for (kind, name) in constants::FIGHTER_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let data = &[MessageType::MappingInfoFighterKinds.into(), kind.as_lua_int().get_int() as u8, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        buf.extend_from_slice(data);
    }
    stream.write(&buf)?;
    Ok(())
}

fn send_stage_constants(mut stream: &TcpStream) -> std::io::Result<()> {
    let mut buf = vec![];
    for (kind, name) in constants::STAGE_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let [kind0, kind1] = (*kind as u16).to_be_bytes();
        let data = &[MessageType::MappingInfoStageKinds.into(), kind0, kind1, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        buf.extend_from_slice(data);
    }
    stream.write(&buf)?;
    Ok(())
}

fn send_fighter_status_kind_constants(mut stream: &TcpStream) -> std::io::Result<()> {
    for (status, fighter, name) in constants::FIGHTER_STATUS_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let [status0, status1] = (status.as_lua_int().get_int() as u16).to_be_bytes();
        let data = &[MessageType::MappingInfoFighterStatusKinds.into(), fighter.as_lua_int().get_int() as u8, status0, status1, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        stream.write(&data)?;

        // Something horrible happens if we don't do this
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    Ok(())
}

fn send_hit_status_constants(mut stream: &TcpStream) -> std::io::Result<()> {
    let mut buf = vec![];
    for (kind, name) in constants::HIT_STATUS_KINDS.iter() {
        let name_bytes = name.as_bytes();
        let data = &[MessageType::MappingInfoHitStatusKinds.into(), kind.as_lua_int().get_int() as u8, name_bytes.len() as u8];
        let data = &[data, name_bytes].concat();
        buf.extend_from_slice(data);
    }
    stream.write(&buf)?;
    Ok(())
}

