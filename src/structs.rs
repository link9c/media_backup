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
    // pub icon:image::DynamicImage
}

impl From<from_slint::ListViewItem> for FileInfo {
    fn from(value: from_slint::ListViewItem) -> Self {
        Self {
            checked: value.checked,
            name: value.name.to_string(),
            size: value.size as u64 * 1024 * 1024,
            modified_time: value.modified_time.to_string(),
            create_time: value.create_time.to_string(),
            // icon:image::DynamicImage::default()
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
        let name_clone = self.name.clone();

        let sp = name_clone.split(".").collect::<Vec<&str>>();
        let file_type = sp[sp.len() - 1];


        // let source_image = {
        //     let mut cat_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        //     cat_path.push("cat.jpg");
        //     image::open(&cat_path).expect("Error loading cat image").into_rgba8()
        // };

        // let icon = self.icon.into_rgb8();
        // let icon_rc = slint::Image::from_rgba8(
        //     slint::SharedPixelBuffer::clone_from_slice(
        //         icon.as_raw(),
        //         icon.width(),
        //         icon.height(),
        //     ),
        // );

        from_slint::ListViewItem {
            checked: self.checked,
            create_time: self.create_time.into(),
            modified_time: self.modified_time.into(),
            name: self.name.into(),
            size: (self.size / 1024 / 1024) as i32,
            size_show: show.into(),
            file_type: file_type.to_string().into(),
            show:false
            // icon:icon_rc
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

// unsafe impl Send for Progress{

// }

impl Default for Progress {
    fn default() -> Self {
        Self {
            total: 0,
            moved: 0,
            status: ProgressStatus::Start,
        }
    }
}
