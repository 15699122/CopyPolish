// python_runtime.rs
// =============================================================================
// PyO3 内嵌 CPython 适配层（路线 B：自定义封装，不依赖 tauri-plugin-python）。
//
// 职责：
//   1. 初始化 CPython 解释器（进程级单例，auto-initialize 已启用）。
//   2. 将项目 `src-python` 目录加入 sys.path。
//   3. 以受控命令调用 `main.py` 桥接模块的
//      format_document / list_rules / enabled_defaults。
//
// 安全边界：只暴露固定业务函数，不向前端开放任意 Python 调用。
// 线程/API 注意（pyo3 0.29）：
//   - 用 Python::try_attach 获取 GIL；解释器由 pyo3 进程级单例管理。
//   - Bound 类型方法为 cast::<T>()（不再使用旧的 downcast）。
//   - PyDict::get_item 返回 PyResult<Option<Bound>>，需 ?.unwrap().extract()。
//   - 链式 call0()?.cast()/call0()?.extract() 会产生临时借用（E0716），
//     一律拆分到中间变量（本文件已在实机编译验证）。
// =============================================================================

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::path::PathBuf;

// 类型定义收敛在 rust_engine 中（Rust 主路径自包含），此处公开复用。
pub use crate::rust_engine::{FormatRequest, RuleMeta};

/// 初始化：把 src-python 加入 sys.path 并预导入桥接模块。
/// 调用时机：应用 setup 阶段一次。
pub fn init(src_python_dir: &PathBuf) -> Result<(), String> {
    // 虽然 Cargo.toml 已启用 pyo3/auto-initialize，但显式初始化能避免
    // 在普通二进制或 Tauri setup 早期调用 try_attach 时返回 None。
    Python::initialize();

    match Python::try_attach(|py| -> PyResult<()> {
        let sys = py.import("sys")?;
        let path_any = sys.getattr("path")?;
        let path = path_any.cast::<PyList>()?;
        let p = src_python_dir.as_path().to_string_lossy().to_string();
        path.append(&p)?;
        let _ = PyModule::import(py, "main")?;
        Ok(())
    }) {
        Some(Ok(())) => Ok(()),
        Some(Err(e)) => Err(format!("Python 初始化失败: {e}")),
        None => Err("Python 解释器不可用（初始化失败）".to_string()),
    }
}

/// format_document(text, enabled) -> String
pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    match Python::try_attach(|py| -> PyResult<String> {
        let bridge = PyModule::import(py, "main")?;
        let fn_any = bridge.getattr("format_document")?;
        let enabled_py = PyList::new(py, req.enabled.as_slice())?;
        let ret_any = fn_any.call1((req.text.as_str(), enabled_py))?;
        let out: String = ret_any.extract()?;
        Ok(out)
    }) {
        Some(Ok(out)) => Ok(out),
        Some(Err(e)) => Err(format!("格式化失败: {e}")),
        None => Err("Python 解释器不可用".to_string()),
    }
}

/// list_rules() -> Vec<RuleMeta>（从桥接返回的 list[dict] 转换）
pub fn get_rules() -> Result<Vec<RuleMeta>, String> {
    match Python::try_attach(|py| -> PyResult<Vec<RuleMeta>> {
        let bridge = PyModule::import(py, "main")?;
        let fn_any = bridge.getattr("list_rules")?;
        let ret_any = fn_any.call0()?;
        let list = ret_any.cast::<PyList>()?;
        let mut out = Vec::with_capacity(list.len());
        for item in list.iter() {
            let d = item.cast::<PyDict>()?;
            let key_any = d.get_item("key")?.unwrap();
            let sec_any = d.get_item("section")?.unwrap();
            let name_any = d.get_item("name")?.unwrap();
            let dis_any = d.get_item("disputed")?.unwrap();
            let def_any = d.get_item("default")?.unwrap();
            out.push(RuleMeta {
                key: key_any.extract::<String>()?,
                section: sec_any.extract::<String>()?,
                name: name_any.extract::<String>()?,
                disputed: dis_any.extract::<bool>()?,
                default: def_any.extract::<bool>()?,
            });
        }
        Ok(out)
    }) {
        Some(Ok(rules)) => Ok(rules),
        Some(Err(e)) => Err(format!("读取规则失败: {e}")),
        None => Err("Python 解释器不可用".to_string()),
    }
}

/// enabled_defaults() -> Vec<String>
pub fn get_enabled_defaults() -> Result<Vec<String>, String> {
    match Python::try_attach(|py| -> PyResult<Vec<String>> {
        let bridge = PyModule::import(py, "main")?;
        let fn_any = bridge.getattr("enabled_defaults")?;
        let ret_any = fn_any.call0()?;
        let list = ret_any.cast::<PyList>()?;
        let mut out = Vec::with_capacity(list.len());
        for item in list.iter() {
            if let Ok(s) = item.extract::<String>() {
                out.push(s);
            }
        }
        Ok(out)
    }) {
        Some(Ok(list)) => Ok(list),
        Some(Err(e)) => Err(format!("读取默认规则失败: {e}")),
        None => Err("Python 解释器不可用".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_runtime() {
        let src_python_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src-python");
        init(&src_python_dir).expect("Python runtime should initialize for tests");
    }

    #[test]
    fn loads_rules_from_embedded_python_bridge() {
        init_test_runtime();

        let rules = get_rules().expect("rules should load through PyO3 bridge");

        assert_eq!(rules.len(), 13);
        assert!(rules
            .iter()
            .any(|rule| rule.key == "中英文之间需要增加空格"));
    }

    #[test]
    fn loads_enabled_defaults_from_embedded_python_bridge() {
        init_test_runtime();

        let defaults = get_enabled_defaults().expect("defaults should load through PyO3 bridge");

        assert_eq!(defaults.len(), 11);
        assert!(defaults.iter().any(|key| key == "中英文之间需要增加空格"));
    }

    #[test]
    fn formats_text_through_embedded_python_bridge() {
        init_test_runtime();

        let formatted = format_text(&FormatRequest {
            text: "在LeanCloud上，花了5000元".to_string(),
            enabled: get_enabled_defaults().expect("defaults should load for format test"),
        })
        .expect("text should format through PyO3 bridge");

        assert_eq!(formatted, "在 LeanCloud 上，花了 5000 元");
    }
}
