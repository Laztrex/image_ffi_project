use crate::error::Result;
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::path::Path;

type ProcessImageFn = unsafe extern "C" fn(
    width: u32,
    height: u32,
    data: *mut u8,
    params: *const std::os::raw::c_char,
);
type ApiVersionFn = unsafe extern "C" fn() -> u32;

const EXPECTED_API_VERSION: u32 = 1;

pub struct Plugin {
    _lib: Library,
    process_fn: ProcessImageFn,
}

impl Plugin {
    pub fn load<P: AsRef<Path>>(plugin_name: &str, plugin_dir: P) -> Result<Self> {
        let lib_path = plugin_dir.as_ref().join(Self::lib_filename(plugin_name));
        if !lib_path.exists() {
            return Err(crate::error::AppError::PluginLoad(format!(
                "Library not found: {}",
                lib_path.display()
            )));
        }
        unsafe {
            let lib = Library::new(lib_path)?;

            // Получаем функцию версии API (без разыменования Symbol)
            let version_fn: Symbol<ApiVersionFn> =
                lib.get(b"plugin_api_version").map_err(|_| {
                    crate::error::AppError::PluginLoad(
                        "Plugin does not export plugin_api_version".into(),
                    )
                })?;
            let version = version_fn(); // вызываем Symbol напрямую
            if version != EXPECTED_API_VERSION {
                return Err(crate::error::AppError::PluginLoad(format!(
                    "Unsupported plugin API version: {}. Expected {}",
                    version, EXPECTED_API_VERSION
                )));
            }

            let process_fn: Symbol<ProcessImageFn> = lib.get(b"process_image")?;
            let process_fn_ptr = *process_fn; // разыменование допустимо для ProcessImageFn (Copy)
            Ok(Plugin {
                _lib: lib,
                process_fn: process_fn_ptr,
            })
        }
    }

    pub fn process(&self, width: u32, height: u32, data: &mut [u8], params: &str) -> Result<()> {
        let params_c = CString::new(params).map_err(|e| {
            crate::error::AppError::ParamParse(format!("params contain null byte: {}", e))
        })?;
        unsafe {
            (self.process_fn)(width, height, data.as_mut_ptr(), params_c.as_ptr());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn lib_filename(name: &str) -> String {
        format!("lib{}.so", name)
    }

    #[cfg(target_os = "windows")]
    fn lib_filename(name: &str) -> String {
        format!("{}.dll", name)
    }

    #[cfg(target_os = "macos")]
    fn lib_filename(name: &str) -> String {
        format!("lib{}.dylib", name)
    }
}
