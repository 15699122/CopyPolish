// Tauri 2 应用装配。默认构建为纯 Rust（rust_engine 主路径，无 Python 依赖）；
// 启用 `python-fallback` feature 时额外编译 PyO3 兜底层并在 setup 阶段初始化解释器。

mod commands;
#[cfg(feature = "python-fallback")]
pub mod python_runtime;
pub mod rust_engine;
mod user_settings;

#[cfg(feature = "python-fallback")]
use std::path::PathBuf;
#[cfg(feature = "python-fallback")]
use tauri::{path::BaseDirectory, Manager};

#[cfg(feature = "python-fallback")]
fn bundled_src_python_dir(resource_main_py: PathBuf) -> Option<PathBuf> {
    resource_main_py.parent().map(PathBuf::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(feature = "python-fallback")]
            {
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
            }
            #[cfg(not(feature = "python-fallback"))]
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::format_text,
            commands::get_rules,
            commands::get_enabled_defaults,
            commands::get_user_settings,
            commands::save_user_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
#[cfg(feature = "python-fallback")]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "python-fallback")]
    fn derives_src_python_dir_from_bundled_main_py_path() {
        let path = PathBuf::from("/tmp/app/resources/src-python/main.py");

        assert_eq!(
            bundled_src_python_dir(path),
            Some(PathBuf::from("/tmp/app/resources/src-python"))
        );
    }
}
