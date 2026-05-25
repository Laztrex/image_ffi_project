use crate::error::Result;
use libloading::{Library, Symbol};
use std::env::consts::DLL_EXTENSION;
use std::ffi::CString;
use std::path::Path;

type ProcessImageFn = unsafe extern "C" fn(
    width: u32,
    height: u32,
    data: *mut u8,
    params: *const std::os::raw::c_char,
) -> i32;
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
            let version = version_fn();
            if version != EXPECTED_API_VERSION {
                return Err(crate::error::AppError::PluginLoad(format!(
                    "Unsupported plugin API version: {}. Expected {}",
                    version, EXPECTED_API_VERSION
                )));
            }

            let process_fn: Symbol<ProcessImageFn> = lib.get(b"process_image")?;
            let process_fn_ptr = *process_fn;
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
        let ret = unsafe { (self.process_fn)(width, height, data.as_mut_ptr(), params_c.as_ptr()) };
        if ret != 0 {
            return Err(crate::error::AppError::PluginLoad(format!(
                "Plugin returned error code {}",
                ret
            )));
        }
        Ok(())
    }

    // Платформозависимое имя библиотеки (с учётом префикса `lib` на Unix)
    #[cfg(target_os = "windows")]
    fn lib_filename(name: &str) -> String {
        format!("{}.{}", name, DLL_EXTENSION)
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn lib_filename(name: &str) -> String {
        format!("lib{}.{}", name, DLL_EXTENSION)
    }
}
