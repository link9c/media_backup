#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
use chrono::prelude::{DateTime, Utc};
use rfd::FileDialog;
use slint::ComponentHandle;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use source::{App, FileAction, ListViewData, ListViewItem};

mod source;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    let app = App::new().unwrap();
    // app.on_clicked(|x|{});
    let handle = app.as_weak();
    app.global::<FileAction>().on_get_file_path(move || {
        let files = FileDialog::new()
            // .add_filter("text", &["txt", "rs"])
            // .add_filter("rust", &["rs", "toml"])
            .set_directory("/")
            .pick_folder();

        match files {
            Some(file_path) => {
                let res = visit_dirs(&file_path);
                let data = res
                    .iter()
                    .map(|x| ListViewItem {
                        checked: false,
                        name: x.name.clone().into(),
                        size: x.size as i32,
                        modified_time: x.modified_time.clone().into(),
                        create_time: x.create_time.clone().into(),
                    })
                    .collect::<Vec<ListViewItem>>();

                let rc_data = Rc::new(slint::VecModel::from(data)).into();
                let file_path = file_path.display().to_string();
                let ui = handle.unwrap();
                ui.global::<FileAction>().set_path(file_path.into());
                ui.global::<ListViewData>().set_select_item(rc_data);
            }
            None => {}
        }
    });
    println!("Hello, world!");
    app.run().unwrap();
}

#[derive(Debug, Clone)]
struct FileInfo {
    name: String,
    size: u64,
    modified_time: String,
    create_time: String,
}

fn visit_dirs(dir: &Path) -> Vec<FileInfo> {
    let mut v: Vec<FileInfo> = vec![];
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    // Here, `entry` is a `DirEntry`.
                    let path = entry.path();
                    if path.is_dir() {
                        // visit_dirs(&path);
                    } else {
                        // println!("{:?}", entry.metadata());
                        let file_size = entry.metadata().unwrap().len();
                        let modified = entry.metadata().unwrap().modified().unwrap();
                        let modified = format_time(&modified);
                        let created = entry.metadata().unwrap().created().unwrap();
                        let created = format_time(&created);
                        let file_name = entry.file_name();

                        // println!("File {} size: {}", file_name.to_string_lossy(), file_size);

                        v.push({
                            FileInfo {
                                name: file_name.to_string_lossy().to_string(),
                                size: file_size,
                                modified_time: modified.into(),
                                create_time: created.into(),
                            }
                        })
                    }
                }
            }
        }
    }
    v
}

fn format_time(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}
