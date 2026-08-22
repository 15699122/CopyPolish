// 参考脚手架升级：Tauri 2 应用装配，注册经由 PyO3 内嵌 CPython 的 command。
// 前端唯一入口是 commands.rs 中定义的受限 command；Python 规则引擎
// 通过 python_runtime.rs 调用（见 Dev_readme「路线 B」对比）。

mod commands;
mod python_runtime;
mod rust_engine;

use std::path::PathBuf;
use tauri::{path::BaseDirectory, Manager};

fn bundled_src_python_dir(resource_main_py: PathBuf) -> Option<PathBuf> {
    resource_main_py.parent().map(PathBuf::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let fallback_src_python_dir =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src-python");
            let src_python_dir = match app
                .path()
                .resolve("src-python/main.py", BaseDirectory::Resource)
                .ok()
                .and_then(bundled_src_python_dir)
            {
                Some(path) if path.join("main.py").is_file() => path,
                Some(path) => {
                    eprintln!(
                        "[pyo3] bundled src-python not found at {}; fallback to {}",
                        path.display(),
                        fallback_src_python_dir.display()
                    );
                    fallback_src_python_dir
                }
                None => {
                    eprintln!(
                        "[pyo3] failed to resolve bundled src-python; fallback to {}",
                        fallback_src_python_dir.display()
                    );
                    fallback_src_python_dir
                }
            };

            // 尽早初始化解释器并暴露桥接错误
            if let Err(e) = python_runtime::init(&src_python_dir) {
                eprintln!("[pyo3] init warning: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::format_text,
            commands::get_rules,
            commands::get_enabled_defaults,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_src_python_dir_from_bundled_main_py_path() {
        let path = PathBuf::from("/tmp/app/resources/src-python/main.py");

        assert_eq!(
            bundled_src_python_dir(path),
            Some(PathBuf::from("/tmp/app/resources/src-python"))
        );
    }
}
