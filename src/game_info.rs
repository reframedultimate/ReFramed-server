pub struct GameInfo {
    match_is_running: bool,

    stage_id: i32,

    p1_entry_id: i32,
    p1_name: String,
    p1_fighter_kind: i32,

    p2_entry_id: i32,
    p2_name: String,
    p2_fighter_kind: i32
}

impl GameInfo {
    pub fn new() -> GameInfo {
        GameInfo {
            match_is_running: false,
            stage_id: -1,
            p1_entry_id: -1,
            p1_name: "".to_string(),
            p1_fighter_kind: -1,
            p2_entry_id: -1,
            p2_name: "".to_string(),
            p2_fighter_kind: -1
        }
    }

    pub fn set_match_end(&mut self) {
        self.match_is_running = false;
        self.p1_entry_id = -1;
        self.p2_entry_id = -1;
    }

    pub fn set_match_start(&mut self) {
        self.match_is_running = true;
    }

    pub fn have_enough_info_to_start_match(&self) -> bool {
        self.p1_entry_id != -1 && self.p2_entry_id != -1 && self.stage_id != -1
    }

    pub fn match_is_running(&self) -> bool {
        self.match_is_running
    }

    pub fn set_player_info(&mut self, entry_id: i32, name: &str, fighter_kind: i32) {
        if self.p1_entry_id == -1 {
            self.p1_entry_id = entry_id;
            self.p1_name = name.to_string();
            self.p1_fighter_kind = fighter_kind;
        } else {            
            self.p2_entry_id = entry_id;
            self.p2_name = name.to_string();
            self.p2_fighter_kind = fighter_kind;
        }
    }

    pub fn set_stage(&mut self, stage_id: i32) {
        self.stage_id = stage_id;
    }

    pub fn get_stage(&self) -> i32 { self.stage_id }

    pub fn p1_entry_id(&self) -> i32 { self.p1_entry_id }
    pub fn p1_name(&self) -> &String { &self.p1_name }
    pub fn p1_fighter_kind(&self) -> i32 { self.p1_fighter_kind }

    pub fn p2_entry_id(&self) -> i32 { self.p2_entry_id }
    pub fn p2_name(&self) -> &String { &self.p2_name }
    pub fn p2_fighter_kind(&self) -> i32 { self.p2_fighter_kind }
}

