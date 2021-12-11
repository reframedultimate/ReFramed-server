pub struct TrainingInfo {
    start_pending: bool,
    is_running: bool,
    stage_id: i32,
    p1_fighter_kind: i32,
    cpu_fighter_kind: i32
}

impl TrainingInfo {
    pub fn new() -> Self {
        Self {
            start_pending: false,
            is_running: false,
            stage_id: -1,
            p1_fighter_kind: -1,
            cpu_fighter_kind: -1
        }
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        self.stage_id = -1;
        self.p1_fighter_kind = -1;
        self.cpu_fighter_kind = -1;
    }

    pub fn set_start_pending(&mut self) {
        self.start_pending = true;
    }

    pub fn is_start_pending(&self) -> bool {
        self.start_pending
    }

    pub fn start(&mut self) {
        self.start_pending = false;
        self.is_running = true;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn set_player_info(&mut self, fighter_kind: i32) {
        self.p1_fighter_kind = fighter_kind;
    }

    pub fn set_cpu_info(&mut self, fighter_kind: i32) {
        self.cpu_fighter_kind = fighter_kind;
    }

    pub fn set_stage(&mut self, stage_id: i32) {
        self.stage_id = stage_id;
    }

    pub fn have_enough_info_to_start(&self) -> bool {
        self.p1_fighter_kind != -1 && self.cpu_fighter_kind != -1 && self.stage_id != -1
    }

    pub fn get_stage(&self) -> i32 {
        self.stage_id
    }

    pub fn p1_fighter_kind(&self) -> i32 {
        self.p1_fighter_kind
    }

    pub fn cpu_fighter_kind(&self) -> i32 {
        self.cpu_fighter_kind
    }
}

