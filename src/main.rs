#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// #![deny(unsafe_code)]
use chrono::prelude::{DateTime, Utc};
use rfd::FileDialog;
use slint::ComponentHandle;
use slint::Model;
use slint::SortModel;
use slint::VecModel;

use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::enums::ProgressStatus;
use crate::structs::from_slint::{App, FileAction, ListItemProgress, ListViewData, ListViewItem};
use crate::structs::{FileInfo, Progress};

mod enums;
mod structs;
static mut PROGRESS_PTR: Progress = Progress {
    moved: 0,
    total: 0,
    status: ProgressStatus::Start,
};

pub fn reset_progress_ptr(){
    unsafe{
        PROGRESS_PTR.moved = 0;
        PROGRESS_PTR.total = 0;
        PROGRESS_PTR.status = ProgressStatus::Start;
    }
    
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    // let progress_ptr = Progress::default();

    // let progress_mutex: Arc<Mutex<Progress>> = Arc::new(Mutex::new(Progress::default()));

    let app = App::new().unwrap();
    // app.on_clicked(|x|{});
    let handle = app.as_weak();
    let handle2 = handle.clone();
    // 定义空列表

    let null_list_rc: Rc<VecModel<ListViewItem>> = Rc::new(VecModel::from(Vec::new()));

    // 定义进度条

    let progress_rc: Rc<VecModel<ListItemProgress>> =
        Rc::new(VecModel::from(vec![ListItemProgress {
            moved: 0,
            total: 0,
            status: 6,
        }]));


    // 代开文件夹 渲染文件列表
    let list_item_copy = null_list_rc.clone();
    let progress_rc_copy = progress_rc.clone();
    app.global::<FileAction>().on_get_file_path(move |idx| {
        let files = FileDialog::new().set_directory("/").pick_folder();

        match files {
            Some(file_path) => {
                let res = visit_dirs(&file_path);

                // let rc_data = Rc::new(slint::VecModel::from(data)).into();
                let file_path = file_path.display().to_string();
                let ui = handle.unwrap();
                println!("----{}", idx);
                if idx == 0 {
                    let data = res
                        .iter()
                        .map(|x| x.clone().into())
                        .collect::<Vec<ListViewItem>>();

                    // data_size.map(|x|)
                    // let data_size = data_size.sum
                    list_item_copy.set_vec(data);
                    ui.global::<FileAction>().set_origin_path(file_path.into());

                    ui.global::<ListViewData>()
                        .set_select_item(list_item_copy.clone().into());
                    ui.global::<ListViewData>()
                        .set_data_size(progress_rc_copy.clone().into());
                } else {
                    ui.global::<FileAction>().set_target_path(file_path.into());
                }
            }
            None => {}
        }
    });
    // 选择所有
    let list_item_copy = null_list_rc.clone();
    app.global::<ListViewData>().on_select_all(move || {
        let ui = handle2.unwrap();
        let count = list_item_copy.row_count();
        let checked = ui.global::<ListViewData>().get_has_select_all();
        ui.global::<ListViewData>().set_has_select_all(!checked);
        for i in 0..count {
            let data = list_item_copy.row_data(i).unwrap();

            list_item_copy.set_row_data(
                i,
                ListViewItem {
                    checked: !checked,
                    name: data.name,
                    size: data.size,
                    modified_time: data.modified_time,
                    create_time: data.create_time,
                    size_show: data.size_show,
                },
            );
        }
    });
    // 排序
    let list_item_copy = null_list_rc.clone();
    app.global::<ListViewData>().on_sort_by(move |idx, asent| {
        let sorted_model = SortModel::new(list_item_copy.clone(), move |lhs, rhs| {
            if asent {
                match idx {
                    0 => lhs.name.to_lowercase().cmp(&rhs.name.to_lowercase()),
                    1 => lhs.size.cmp(&rhs.size),
                    2 => lhs.create_time.cmp(&rhs.create_time),
                    _ => lhs.modified_time.cmp(&rhs.modified_time),
                }
            } else {
                match idx {
                    0 => rhs.name.to_lowercase().cmp(&lhs.name.to_lowercase()),
                    1 => rhs.size.cmp(&lhs.size),
                    2 => rhs.create_time.cmp(&lhs.create_time),
                    _ => rhs.modified_time.cmp(&lhs.modified_time),
                }
            }
        });
        let sorted_item = sorted_model.iter().collect::<Vec<ListViewItem>>();
        list_item_copy.set_vec(sorted_item);
    });
    // 备份文件
    let list_item_copy = null_list_rc.clone();
    let progress_rc_copy = progress_rc.clone();
    app.global::<FileAction>()
        .on_copy_file(move |origin_path, target_path| {
            let items = list_item_copy
                .iter()
                .map(|x| FileInfo::from(x))
                .collect::<Vec<FileInfo>>();

            // println!("{:?}",items);
            let data_size = items
                .iter()
                .map(|x| {
                    if x.checked {
                        (x.size / 1024) as i32
                    } else {
                        0 as i32
                    }
                })
                .sum::<i32>();
            println!("data-size:{}", data_size);
            progress_rc_copy.set_row_data(
                0,
                ListItemProgress {
                    moved: 0,
                    total: data_size,
                    status: 0,
                },
            );

            let count = items.iter().count();

            if count > 0 {
                thread::Builder::new()
                    .spawn(move || {
                        // unsafe {
                        //     PROGRESS_PTR.moved = 0;
                        //     PROGRESS_PTR.total = 0;
                        //     PROGRESS_PTR.status = ProgressStatus::Start;
                        // };

                        reset_progress_ptr();

                        for (i, each) in items.iter().enumerate() {
                            // let pr_copy = pr.clone();
                            let o_path = format!("{}/{}", origin_path.to_string(), each.name);
                            let t_path = format!("{}/{}", target_path.to_string(), each.name);
                            println!("{},{}", i, count);
                            let state = if i == count {
                                ProgressStatus::Finish
                            } else {
                                ProgressStatus::Continue
                            };
                            if each.checked {
                                copy_file_buffer(&o_path, &t_path, state);
                            }
                        }
                    })
                    .unwrap();
            }
        });
    // let progress_mutex_2 = progress_mutex.clone();
    let progress_rc_copy = progress_rc.clone();
    app.global::<FileAction>()
        .on_update_progress_status(move |x| unsafe {
            let state = ProgressStatus::from_num(x);
            PROGRESS_PTR.status = state;
            if state == ProgressStatus::Exit{
                progress_rc_copy.set_row_data(
                    0,
                    ListItemProgress {
                        moved: 0,
                        total: 0,
                        status: 0,
                    },
                );
            }
        });
    // let progress_mutex_3 = progress_mutex.clone();
    let progress_rc_copy = progress_rc.clone();
    let tick1 = slint::Timer::default();

    tick1.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs_f32(0.2),
        move || {
            let total = progress_rc_copy.row_data(0);
            if let Some(total) = total {
                // println!("{:?}",total);
                let data: ListItemProgress = {
                    unsafe {
                        PROGRESS_PTR.total = (total.total * 1024) as u64;
                        println!("{:?}", PROGRESS_PTR);
                        PROGRESS_PTR.into()
                    }
                };

                progress_rc_copy.set_row_data(0, data);
            }
        },
    );

    app.run().unwrap();
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
                                checked: false,
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

fn copy_file_buffer(
    filepath: &str,
    target_filepath: &str,
    state: ProgressStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    const BUFFER_LEN: usize = 512;
    let mut buffer = [0u8; BUFFER_LEN];
    let mut file = File::open(filepath)?;
    let target_file = File::create(target_filepath)?;
    let mut target_bw = BufWriter::new(target_file);

    loop {
        unsafe {
            match PROGRESS_PTR.status {
                ProgressStatus::Start | ProgressStatus::Continue => {
                    let read_count = file.read(&mut buffer)?;
                    target_bw.write(&buffer[..read_count])?;
                    PROGRESS_PTR.moved = PROGRESS_PTR.moved + BUFFER_LEN as u64;
                    if read_count != BUFFER_LEN {
                        target_bw.flush()?;
                        PROGRESS_PTR.status = state;
                        break;
                    }
                }

                ProgressStatus::Stop => {
                    thread::sleep(Duration::new(1, 0));
                }

                ProgressStatus::Exit => {
                    break;
                }
                ProgressStatus::Finish => {}
            }
        }
    }
    Ok(())
}
