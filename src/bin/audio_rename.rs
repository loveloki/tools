use lofty::prelude::{Accessor, TaggedFileExt};
use lofty::probe::Probe;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use walkdir::WalkDir;

fn main() {
    println!("=== 通用音频批量重命名工具 ===");
    println!("递归扫描当前目录及子目录下的音频文件，根据元数据重命名文件");
    println!("支持格式: m4a, mp3, flac, wav, ogg, aac, aiff, wma, ape, opus, mp4");
    println!("正在递归扫描当前目录及子目录下的音频文件...\n");

    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("错误: 无法获取当前目录: {}", e);
            wait_for_enter();
            return;
        }
    };

    let mut count_success = 0;
    let mut count_skip = 0;
    let mut count_error = 0;

    let supported_extensions = [
        "m4a", "mp3", "flac", "wav", "ogg", "aac", "aiff", "wma", "ape", "opus", "mp4",
    ];

    for entry in WalkDir::new(&current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                if supported_extensions.contains(&ext_str.as_str()) {
                    match process_file(path, &ext_str) {
                        Ok(renamed) => {
                            if renamed {
                                count_success += 1;
                            } else {
                                count_skip += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ 处理文件 {:?} 失败: {}",
                                path.file_name().unwrap_or_default(),
                                e
                            );
                            count_error += 1;
                        }
                    }
                }
            }
        }
    }

    println!("\n=== 执行完毕 ===");
    println!("✅ 成功: {}", count_success);
    println!("⏭️  跳过: {}", count_skip);
    println!("❌ 失败: {}", count_error);

    wait_for_enter();
}

fn process_file(path: &Path, extension: &str) -> Result<bool, String> {
    let old_filename = path
        .file_name()
        .ok_or("无法获取文件名")?
        .to_string_lossy()
        .to_string();

    // 读取元数据
    let tagged_file = Probe::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?
        .read()
        .map_err(|e| format!("无法读取元数据: {}", e))?;

    let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        None => tagged_file.first_tag().ok_or("文件中没有元数据标签")?,
    };

    let title = tag
        .title()
        .as_deref()
        .unwrap_or("Unknown Title")
        .trim()
        .to_string();
    let track_number = tag.track();

    // 格式化新文件名 (保留原始扩展名)
    let new_filename_str = match track_number {
        Some(track) => format!("{:02} - {}.{}", track, title, extension),
        None => format!("{}.{}", title, extension),
    };

    // 清洗非法字符
    let safe_filename_str = sanitize_filename(&new_filename_str);

    if old_filename == safe_filename_str {
        println!("⏭️  跳过 (文件名已正确): {}", old_filename);
        return Ok(false);
    }

    let new_path = path.with_file_name(&safe_filename_str);

    // 防覆盖检查
    if new_path.exists() {
        println!(
            "⚠️  跳过 (目标文件已存在): {} -> {}",
            old_filename, safe_filename_str
        );
        return Ok(false);
    }

    // 执行重命名
    fs::rename(path, &new_path).map_err(|e| format!("重命名失败: {}", e))?;
    println!("✅ 重命名: {} -> {}", old_filename, safe_filename_str);

    Ok(true)
}

fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['/', ':', '?', '*', '\\', '<', '>', '|', '"'];
    filename
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect()
}

fn wait_for_enter() {
    println!("\n按回车键退出...");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}
