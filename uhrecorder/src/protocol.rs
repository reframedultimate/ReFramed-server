use std::net::TcpStream;
use std::io::Write;
use num_enum::IntoPrimitive;
use smash::lib::lua_const::*;

#[derive(IntoPrimitive)]
#[repr(u8)]
enum MessageType {
    FighterKind,
	StageKind
}

pub trait Protocol {
    fn send_constants(client: &mut TcpStream) -> Result<(), std::io::Error> {
		send_fighter_constants(client)?;
		send_stage_constants(client)?;
		Ok(())
    }
}

fn send_fighter_constants(client: &mut TcpStream) -> Result<(), std::io::Error> {
	for (kind, name) in [
		(*FIGHTER_KIND_MARIO,        String::from("Mario")),
		(*FIGHTER_KIND_DONKEY,       String::from("Donkey Kong")),
		(*FIGHTER_KIND_LINK,         String::from("Link")),
		(*FIGHTER_KIND_SAMUS,        String::from("Samus")),
		(*FIGHTER_KIND_YOSHI,        String::from("Yoshi")),
		(*FIGHTER_KIND_KIRBY,        String::from("Kirby")),
		(*FIGHTER_KIND_FOX,          String::from("Fox")),
		(*FIGHTER_KIND_PIKACHU,      String::from("Pikachu")),
		(*FIGHTER_KIND_LUIGI,        String::from("Luigi")),
		(*FIGHTER_KIND_NESS,         String::from("Ness")),
		(*FIGHTER_KIND_CAPTAIN,      String::from("Captain Falcon")),
		(*FIGHTER_KIND_PURIN,        String::from("Jigglypuff")),
		(*FIGHTER_KIND_PEACH,        String::from("Peach")),
		(*FIGHTER_KIND_KOOPA,        String::from("Bowser")),
		(*FIGHTER_KIND_POPO,         String::from("Ice Climbers")),
		(*FIGHTER_KIND_NANA,         String::from("Ice Climbers")),
		(*FIGHTER_KIND_SHEIK,        String::from("Sheik")),
		(*FIGHTER_KIND_ZELDA,        String::from("Zelda")),
		(*FIGHTER_KIND_MARIOD,       String::from("Doctor Mario")),
		(*FIGHTER_KIND_PICHU,        String::from("Pichu")),
		(*FIGHTER_KIND_FALCO,        String::from("Falco")),
		(*FIGHTER_KIND_MARTH,        String::from("Marth")),
		(*FIGHTER_KIND_YOUNGLINK,    String::from("Young Link")),
		(*FIGHTER_KIND_GANON,        String::from("Ganondorf")),
		(*FIGHTER_KIND_MEWTWO,       String::from("Mewto")),
		(*FIGHTER_KIND_ROY,          String::from("Roy")),
		(*FIGHTER_KIND_GAMEWATCH,    String::from("Mr. Game & Watch")),
		(*FIGHTER_KIND_METAKNIGHT,   String::from("Meta Knight")),
		(*FIGHTER_KIND_PIT,          String::from("Pit")),
		(*FIGHTER_KIND_SZEROSUIT,    String::from("Zero Suit Samus")),
		(*FIGHTER_KIND_WARIO,        String::from("Wario")),
		(*FIGHTER_KIND_SNAKE,        String::from("Snake")),
		(*FIGHTER_KIND_IKE,          String::from("Ike")),
		(*FIGHTER_KIND_PZENIGAME,    String::from("Squirtle")),
		(*FIGHTER_KIND_PFUSHIGISOU,  String::from("Ivysaur")),
		(*FIGHTER_KIND_PLIZARDON,    String::from("Charizard")),
		(*FIGHTER_KIND_DIDDY,        String::from("Diddy Kong")),
		(*FIGHTER_KIND_LUCAS,        String::from("Lucas")),
		(*FIGHTER_KIND_SONIC,        String::from("Sonic")),
		(*FIGHTER_KIND_DEDEDE,       String::from("King Dedede")),
		(*FIGHTER_KIND_PIKMIN,       String::from("Olimar")),
		(*FIGHTER_KIND_LUCARIO,      String::from("Lucario")),
		(*FIGHTER_KIND_ROBOT,        String::from("R.O.B.")),
		(*FIGHTER_KIND_TOONLINK,     String::from("Toon Link")),
		(*FIGHTER_KIND_WOLF,         String::from("Wolf")),
		(*FIGHTER_KIND_MURABITO,     String::from("Villager")),
		(*FIGHTER_KIND_ROCKMAN,      String::from("Mega Man")),
		(*FIGHTER_KIND_WIIFIT,       String::from("Wii Fit Trainer")),
		(*FIGHTER_KIND_ROSETTA,      String::from("Rosalina")),
		(*FIGHTER_KIND_LITTLEMAC,    String::from("Little Mac")),
		(*FIGHTER_KIND_GEKKOUGA,     String::from("Greninja")),
		(*FIGHTER_KIND_PALUTENA,     String::from("Palutena")),
		(*FIGHTER_KIND_PACMAN,       String::from("Pac-Man")),
		(*FIGHTER_KIND_REFLET,       String::from("Robin")),
		(*FIGHTER_KIND_SHULK,        String::from("Shulk")),
		(*FIGHTER_KIND_KOOPAJR,      String::from("Bowser Jr")),
		(*FIGHTER_KIND_DUCKHUNT,     String::from("Duck Hunt Duo")),
		(*FIGHTER_KIND_RYU,          String::from("Ryu")),
		(*FIGHTER_KIND_CLOUD,        String::from("Cloud")),
		(*FIGHTER_KIND_KAMUI,        String::from("Corrin")),
		(*FIGHTER_KIND_BAYONETTA,    String::from("Bayonetta")),
		(*FIGHTER_KIND_INKLING,      String::from("Inkling")),
		(*FIGHTER_KIND_RIDLEY,       String::from("Ridley")),
		(*FIGHTER_KIND_SIMON,        String::from("Simon")),
		(*FIGHTER_KIND_KROOL,        String::from("K. Rool")),
		(*FIGHTER_KIND_SHIZUE,       String::from("Isabelle")),
		(*FIGHTER_KIND_GAOGAEN,      String::from("Incineroar")),
		(*FIGHTER_KIND_PACKUN,       String::from("Piranha Plant")),
		(*FIGHTER_KIND_JACK,         String::from("")),
		(*FIGHTER_KIND_BRAVE,        String::from("")),
		(*FIGHTER_KIND_BUDDY,        String::from("")),
		(*FIGHTER_KIND_DOLLY,        String::from("")),
		(*FIGHTER_KIND_MASTER,       String::from("")),
		(*FIGHTER_KIND_TANTAN,       String::from("")),
		(*FIGHTER_KIND_PICKEL,       String::from("")),
		(*FIGHTER_KIND_EDGE,         String::from("")),
		(*FIGHTER_KIND_MIIFIGHTER,   String::from("Mii Fighter")),
		(*FIGHTER_KIND_MIISWORDSMAN, String::from("Mii Swordfighter")),
		(*FIGHTER_KIND_MIIGUNNER,    String::from("Mii Gunner")),
		(*FIGHTER_KIND_SAMUSD,       String::from("Dark Samus")),
		(*FIGHTER_KIND_DAISY,        String::from("Daisy")),
		(*FIGHTER_KIND_LUCINA,       String::from("Lucina")),
		(*FIGHTER_KIND_CHROM,        String::from("Chrom")),
		(*FIGHTER_KIND_PITB,         String::from("Pit")),
		(*FIGHTER_KIND_KEN,          String::from("Ken")),
		(*FIGHTER_KIND_RICHTER,      String::from("Richter")),
		(*FIGHTER_KIND_KOOPAG,       String::from("")),
		(*FIGHTER_KIND_MIIENEMYF,    String::from("")),
		(*FIGHTER_KIND_MIIENEMYS,    String::from("")),
		(*FIGHTER_KIND_MIIENEMYG,    String::from("")),
	].iter() {
		let name_bytes = name.as_bytes();
		let part = &[MessageType::FighterKind.into(), *kind as u8, name_bytes.len() as u8];
		let data = &[part, name_bytes].concat();
		client.write(data)?;
	}
	Ok(())
}

fn send_stage_constants(client: &mut TcpStream) -> Result<(), std::io::Error> {
	for (kind, name) in [
		(0,   String::from("Battlefield")),
		(1,   String::from("Final Destination")),  // BF Omega form
		(3,   String::from("Final Destination")),
		(4,   String::from("Battlefield")),  // FD battlefield form
		(5,   String::from("Peach's Castle")),
		(44,  String::from("Yoshi's Story")),
		(89,  String::from("Lylat Cruise")),
		(95,  String::from("Smashville")),
		(107, String::from("Pokemon Stadium 2")),
		(242, String::from("Kalos Pokemon League")),
		(257, String::from("Town and City")),
		(347, String::from("Small Battlefield")),
	].iter() {
		let name_bytes = name.as_bytes();
		let part = &[MessageType::StageKind.into(), *kind as u8, name_bytes.len() as u8];
		let data = &[part, name_bytes].concat();
		client.write(data)?;
	}
	Ok(())
}

