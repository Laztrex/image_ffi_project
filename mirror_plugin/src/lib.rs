use log::warn;
use serde::Deserialize;
use std::ffi::CStr;
use std::slice;

#[derive(Deserialize)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool,
}

/// Экспортируемая версия API плагина.
#[no_mangle]
pub extern "C" fn plugin_api_version() -> u32 {
    1
}

/// Отражает изображение.
/// 
/// # Safety
/// `rgba_data` должен указывать на валидный буфер размером не менее `width * height * 4` байт.
/// Буфер должен быть выровнен и не пересекаться с другими данными.
/// Параметры `width`, `height` должны соответствовать реальному размеру изображения.
/// 
/// Возвращает 0 при успехе, ненулевое значение при ошибке.
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::os::raw::c_char,
) -> i32 {
    if rgba_data.is_null() {
        warn!("process_image: null pointer to rgba_data");
        return 1;
    }

    let len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|p| p.checked_mul(4))
    {
        Some(l) => l,
        None => {
            warn!("Buffer size overflow: {}x{}x4", width, height);
            return 2;
        }
    };
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    let params_str = if params.is_null() {
        "{}"
    } else {
        match unsafe { CStr::from_ptr(params).to_str() } {
            Ok(s) => s,
            Err(_) => {
                warn!("Invalid UTF-8 in params string");
                "{}"
            }
        }
    };

    let mirror_params: MirrorParams = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to parse JSON params '{}': {}", params_str, e);
            MirrorParams {
                horizontal: false,
                vertical: false,
            }
        }
    };

    let (w, h) = (width as usize, height as usize);

    // Горизонтальное отражение
    if mirror_params.horizontal {
        for row in 0..h {
            let row_start = row * w * 4;
            let row_data = &mut data[row_start..row_start + w * 4];
            for col in 0..w / 2 {
                let left = col * 4;
                let right = (w - 1 - col) * 4;
                for i in 0..4 {
                    row_data.swap(left + i, right + i);
                }
            }
        }
    }

    // Вертикальное отражение – меняем строки целиком через split_at_mut
    if mirror_params.vertical {
        let row_size = w * 4;
        for row in 0..h / 2 {
            let top_start = row * row_size;
            let bottom_start = (h - 1 - row) * row_size;
            // Убедимся, что top_start < bottom_start
            if top_start < bottom_start {
                let (first, second) = data.split_at_mut(bottom_start);
                let top_slice = &mut first[top_start..top_start + row_size];
                let bottom_slice = &mut second[..row_size];
                top_slice.swap_with_slice(bottom_slice);
            } else {
                let (first, second) = data.split_at_mut(top_start);
                let bottom_slice = &mut first[bottom_start..bottom_start + row_size];
                let top_slice = &mut second[..row_size];
                bottom_slice.swap_with_slice(top_slice);
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_horizontal_mirror_2x2() {
        let mut data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let width = 2;
        let height = 2;
        let params = CString::new(r#"{"horizontal":true,"vertical":false}"#).unwrap();

        unsafe {
            process_image(width, height, data.as_mut_ptr(), params.as_ptr());
        }

        let expected: Vec<u8> = vec![5, 6, 7, 8, 1, 2, 3, 4, 13, 14, 15, 16, 9, 10, 11, 12];
        assert_eq!(data, expected);
    }

    #[test]
    fn test_vertical_mirror_2x2() {
        let mut data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let width = 2;
        let height = 2;
        let params = CString::new(r#"{"horizontal":false,"vertical":true}"#).unwrap();

        unsafe {
            process_image(width, height, data.as_mut_ptr(), params.as_ptr());
        }

        let expected: Vec<u8> = vec![9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(data, expected);
    }
}
