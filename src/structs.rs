pub mod from_slint {
    slint::include_modules!();
}

use crate::enums::ProgressStatus;
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub checked: bool,
    pub name: String,
    pub size: u64,
    pub modified_time: String,
    pub create_time: String,
}

impl From<from_slint::ListViewItem> for FileInfo {
    fn from(value: from_slint::ListViewItem) -> Self {
        Self {
            checked: value.checked,
            name: value.name.to_string(),
            size: value.size as u64 * 1024 * 1024,
            modified_time: value.modified_time.to_string(),
            create_time: value.create_time.to_string(),
        }
    }
}

impl Into<from_slint::ListViewItem> for FileInfo {
    fn into(self) -> from_slint::ListViewItem {
        let show = if self.size < 1024 {
            format!("{} b", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{} kb", self.size / 1024)
        } else {
            format!("{} mb", self.size / 1024 / 1024)
        };
        from_slint::ListViewItem {
            checked: self.checked,
            create_time: self.create_time.into(),
            modified_time: self.modified_time.into(),
            name: self.name.into(),
            size: (self.size / 1024 / 1024) as i32,
            size_show: show.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Progress {
    pub total: u64,
    pub moved: u64,
    pub status: ProgressStatus,
}

impl Into<from_slint::ListItemProgress> for Progress {
    fn into(self) -> from_slint::ListItemProgress {
        
        from_slint::ListItemProgress {
            moved: (self.moved / 1024) as i32,
            status: self.status.to_num(),
            total: (self.total / 1024) as i32,
        }
    }
}
