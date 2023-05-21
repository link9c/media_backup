#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// #![deny(unsafe_code)]

use rfd::FileDialog;
use slint::ComponentHandle;
use slint::Model;
use slint::SortModel;
use slint::VecModel;

use std::rc::Rc;
use std::thread;

use crate::enums::ProgressStatus;
use crate::enums::StaticVarsType;
use crate::file_opt::copy_file_buffer;
use crate::file_opt::visit_dirs;
use crate::static_vars::get_progress_ptr;
use crate::static_vars::update_progress_ptr;
use crate::structs::from_slint::{App, FileAction, ListItemProgress, ListViewData, ListViewItem};
use crate::structs::FileInfo;

mod enums;
mod file_opt;
mod static_vars;
mod structs;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
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

    // 打开文件夹 渲染文件列表
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
                    file_type: data.file_type,
                    show: data.show, // icon:data.icon
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
    // 查询文件
    let list_item_copy = null_list_rc.clone();
    app.global::<ListViewData>().on_search_by_name(move |text| {
        let mut filter_item = list_item_copy
            .iter()
            .map(|mut x| {
                if text.len() > 0 {
                    if x.name.contains(&text.to_string()) {
                        x.show = true;
                        x.checked= true;
                    } else {
                        x.show = false;
                        
                    }
                } else {
                    x.show = false;
                 
                }

                x
            })
            .collect::<Vec<ListViewItem>>();

        filter_item.sort_by_key(|x| !x.show);

        list_item_copy.set_vec(filter_item);
    });
    // 备份文件
    let list_item_copy = null_list_rc.clone();
    let progress_rc_copy = progress_rc.clone();
    app.global::<FileAction>()
        .on_copy_file(move |origin_path, target_path| {
            let items = list_item_copy
                .iter()
                .filter_map(|x| {
                    if x.checked {
                        Some(FileInfo::from(x))
                    } else {
                        None
                    }
                })
                .collect::<Vec<FileInfo>>();

            // println!("{:?}",items);
            let data_size = items.iter().map(|x| (x.size / 1024) as i32).sum::<i32>();
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
                        update_progress_ptr(
                            StaticVarsType::Update(0),
                            StaticVarsType::Update((data_size as u64) * 1024),
                            StaticVarsType::Update(ProgressStatus::Start),
                        );

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
                                let _ = copy_file_buffer(&o_path, &t_path, state);
                            }
                        }
                    })
                    .unwrap();
            }
        });
    app.global::<FileAction>()
        .on_update_progress_status(move |x| {
            let state = ProgressStatus::from_num(x);
            // PROGRESS_PTR.status = state;
            update_progress_ptr(
                StaticVarsType::Keep,
                StaticVarsType::Keep,
                StaticVarsType::Update(state),
            );
        });

    // 获取进度信息
    let progress_rc_copy = progress_rc.clone();
    let tick1 = slint::Timer::default();
    tick1.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs_f32(0.2),
        move || {
            let data: ListItemProgress = get_progress_ptr().clone().into();

            progress_rc_copy.set_row_data(0, data);
        },
    );

    app.run().unwrap();
}
