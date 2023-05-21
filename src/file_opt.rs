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
                        let name = file_name.to_string_lossy().to_string();

                        // let icon = get_file_type_icon(name.clone());
                        // println!("File {} size: {}", file_name.to_string_lossy(), file_size);

                        v.push({
                            FileInfo {
                                name: name,
                                size: file_size,
                                modified_time: modified.into(),
                                create_time: created.into(),
                                checked: false,
                                // icon: icon,
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

pub fn copy_file_buffer(
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
            ProgressStatus::Finish => {}
        }
    }
    Ok(())
}

fn get_file_type_icon(name: String) -> image::DynamicImage {
    let sp = name.split(".").collect::<Vec<&str>>();
    let file_type = sp[sp.len() - 1];
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let source_image_path = match file_type {
        "ai" | "eps" => {
            path.push("ui/images/file_type/ae.png");
            path
        }
        "doc" | "docx" => {
            path.push("images/file_type/doc rtf.png");
            path
        }

        "html" | "htm" => {
            path.push("ui/images/file_type/html htm IE.png");
            path
        }

        "mpeg" | "avi" | "wav" | "ogg" | "mp3" | "mp4" | "mkv" => {
            path.push("ui/images/file_type/mpeg avi wav ogg mp3.png");
            path
        }

        "pdf" => {
            path.push("ui/images/file_type/pdf.png");
            path
        }
        "ppt" => {
            path.push("ui/images/file_type/ppt.png");
            path
        }
        "torrent" => {
            path.push("ui/images/file_type/torrent.png");
            path
        }

        "zip" | "rar" | "7z" => {
            path.push("ui/images/file_type/zip rar.png");
            path
        }

        "xls" | "xlsx" | "csv" | "xlsm" => {
            path.push("ui/images/file_type/xls.png");
            path
        }

        _ => {
            path.push("ui/images/cat.jpg");
            path
        }
    };
    println!("path{:?}", source_image_path);
    image::open(source_image_path).expect("Error loading cat image")
}
