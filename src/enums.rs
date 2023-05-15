#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressStatus {
    Start,
    Stop,
    Continue,
    Exit,
    Finish,
}

impl Default for ProgressStatus {
    fn default() -> Self {
        ProgressStatus::Start
    }
}

impl ProgressStatus {
    pub fn to_num(self) -> i32 {
        let num = match self {
            ProgressStatus::Start => 0,
            ProgressStatus::Stop => 1,
            ProgressStatus::Continue => 2,
            ProgressStatus::Exit => 3,
            ProgressStatus::Finish => 4,
        };
        num
    }

    pub fn from_num(idx: i32) -> Self {
        match idx {
            0 => ProgressStatus::Start,
            1 => ProgressStatus::Stop,
            2 => ProgressStatus::Continue,
            3 => ProgressStatus::Exit,
            4 => ProgressStatus::Finish,
            _ => ProgressStatus::Finish
        }
    }
}
