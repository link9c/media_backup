// 文件操作

use crate::enums::ProgressStatus;
use crate::enums::StaticVarsType;
use crate::static_vars::get_progress_ptr;
use crate::static_vars::update_progress_ptr;
use crate::structs::FileInfo;

use chrono::prelude::{DateTime, Utc};
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn visit_dirs(dir: &Path) -> Vec<FileInfo> {
    let mut v: Vec<FileInfo> = vec![];
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    // Here, `entry` is a `DirEntry`.
                    let path = entry.path();
                    let file_size = entry.metadata().unwrap().len();
                    let modified = entry.metadata().unwrap().modified().unwrap();
                    let modified = format_time(&modified);
                    let created = entry.metadata().unwrap().created().unwrap();
                    let created = format_time(&created);
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy().to_string();
                    let dirs = if path.is_dir() {
                        // visit_dirs(&path);
                        true
                    } else {
                        // println!("{:?}", entry.metadata());

                        // let icon = get_file_type_icon(name.clone());
                        // println!("File {} size: {}", file_name.to_string_lossy(), file_size);
                        false
                    };

                    v.push({
                        FileInfo {
                            name: name,
                            size: file_size,
                            modified_time: modified.into(),
                            create_time: created.into(),
                            checked: false,
                            is_dir: dirs, // icon: icon,
                        }
                    })
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
    filepath: &Path,
    target_filepath: &Path,
    state: ProgressStatus,
) -> std::io::Result<()> {
    const BUFFER_LEN: usize = 512;
    let mut buffer = [0u8; BUFFER_LEN];
    let mut file = File::open(filepath)?;
    let target_file = File::create(target_filepath)?;
    let mut target_bw = BufWriter::new(target_file);

    loop {
        let ptr = get_progress_ptr().clone();
        // println!("sssss{:?}",ptr);
        match ptr.status {
            ProgressStatus::Start | ProgressStatus::Continue => {
                let read_count = file.read(&mut buffer)?;
                target_bw.write(&buffer[..read_count])?;
                // ptr.moved = ptr.moved + BUFFER_LEN as u64;

                update_progress_ptr(
                    StaticVarsType::Update(ptr.moved + BUFFER_LEN as u64),
                    StaticVarsType::Keep,
                    StaticVarsType::Keep,
                );
                if read_count != BUFFER_LEN {
                    target_bw.flush()?;
                    update_progress_ptr(
                        StaticVarsType::Keep,
                        StaticVarsType::Keep,
                        StaticVarsType::Update(state),
                    );
                    break;
                }
            }

            ProgressStatus::Stop => {
                thread::sleep(Duration::new(1, 0));
            }

            ProgressStatus::Exit => {
                update_progress_ptr(
                    StaticVarsType::Update(0),
                    StaticVarsType::Update(0),
                    StaticVarsType::Keep,
                );
                break;
            }
            ProgressStatus::Finish => {break;}
        }
    }
    Ok(())
}

pub fn copy_folder_content(
    filepath: &Path,
    target_filepath: &Path,
    state: ProgressStatus,
) -> std::io::Result<()> {
    // let src = Path::new(filepath);
    // let dst = Path::new(target_filepath);
    if filepath.is_dir() {
        let _ = fs::create_dir(target_filepath);
        if let Ok(entries) = fs::read_dir(filepath) {
            
            for entry in entries {
                if let Ok(entry) = entry {
                    // Here, `entry` is a `DirEntry`.
                    
                    let path = entry.path();
                    // println!("{:?}", entry);
                    let file_name = entry.file_name();
                    let dsr = target_filepath.join(file_name.to_string_lossy().to_string());
                    if path.is_dir() {
                        // println!("dir:{:?}-->{:?}", path, dsr);
                        let _ = fs::create_dir(dsr.clone());
                        copy_folder_content(&path, &dsr, state)?;
                    } else {
                        // println!("copy1:{:?}-->{:?}", filepath, dsr);
                        copy_file_buffer(&path, &dsr, state)?;
                    };
                }
            }
        }
    } else {
        // println!("copy2:{:?}-->{:?}", filepath, target_filepath);
        copy_file_buffer(filepath, target_filepath, state)?;
    }
    Ok(())
}
