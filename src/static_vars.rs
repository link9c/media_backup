use crate::enums::ProgressStatus;
use crate::enums::StaticVarsType;

use crate::structs::Progress;

pub static mut PROGRESS_PTR: Progress = Progress {
    moved: 0,
    total: 0,
    status: ProgressStatus::Start,
};
/// 更新全局变量Progress
///
/// StaticVarsType::Keep => 保持不变
///
/// StaticVarsType::Update(T) => 更新为T
/// ```
///  unsafe {
///     PROGRESS_PTR.moved = moved;
///     PROGRESS_PTR.total = total;
///     PROGRESS_PTR.status = status.unwrap_or(ProgressStatus::Start);
/// }
/// ```
pub fn update_progress_ptr(
    moved: StaticVarsType<u64>,
    total: StaticVarsType<u64>,
    status: StaticVarsType<ProgressStatus>,
) {
    unsafe {
        match moved {
            StaticVarsType::Update(t) => {
                PROGRESS_PTR.moved = t;
            }
            StaticVarsType::Keep => {}
        };

        match total {
            StaticVarsType::Update(t) => {
                PROGRESS_PTR.total = t;
            }
            StaticVarsType::Keep => {}
        };

        match status {
            StaticVarsType::Update(t) => {
                PROGRESS_PTR.status = t;
            }
            StaticVarsType::Keep => {}
        };
    }
}

pub fn get_progress_ptr() -> Progress {
    unsafe { PROGRESS_PTR }
}
