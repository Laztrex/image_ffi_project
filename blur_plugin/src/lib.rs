use log::warn;
use serde::Deserialize;
use std::ffi::CStr;
use std::slice;

#[derive(Deserialize)]
struct BlurParams {
    radius: u32,
    iterations: u32,
}

#[no_mangle]
pub extern "C" fn plugin_api_version() -> u32 {
    1
}

/// Размывает изображение без лишних копий (использует временный буфер).
/// # Safety
/// `rgba_data` должен указывать на валидный буфер размером не менее `width * height * 4`.
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::os::raw::c_char,
) {
    let len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|p| p.checked_mul(4))
    {
        Some(l) => l,
        None => {
            warn!("Buffer size overflow: {}x{}x4", width, height);
            return;
        }
    };

    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    let params_str = unsafe { CStr::from_ptr(params).to_str().unwrap_or("{}") };

    let blur_params: BlurParams = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to parse JSON params '{}': {}", params_str, e);

            BlurParams {
                radius: 1,
                iterations: 1,
            }
        }
    };

    let (w, h) = (width as usize, height as usize);
    let radius = blur_params.radius as usize;

    let mut temp = vec![0u8; len];

    for _ in 0..blur_params.iterations {
        blur_pass(data, &mut temp, w, h, radius);
        data.copy_from_slice(&temp);
    }
}

pub fn blur_pass(src: &[u8], dst: &mut [u8], width: usize, height: usize, radius: usize) {
    const BYTES_PER_PIXEL: usize = 4;

    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0u32;
            let mut sum_g = 0u32;
            let mut sum_b = 0u32;
            let mut count = 0u32;

            for dy in -(radius as i32)..=(radius as i32) {
                let ny = y as i32 + dy;

                if ny < 0 || ny >= height as i32 {
                    continue;
                }

                for dx in -(radius as i32)..=(radius as i32) {
                    let nx = x as i32 + dx;

                    if nx < 0 || nx >= width as i32 {
                        continue;
                    }

                    let idx = ((ny * width as i32 + nx) as usize) * BYTES_PER_PIXEL;

                    sum_r += src[idx] as u32;
                    sum_g += src[idx + 1] as u32;
                    sum_b += src[idx + 2] as u32;

                    count += 1;
                }
            }

            let out_idx = (y * width + x) * BYTES_PER_PIXEL;

            if let (Some(r), Some(g), Some(b)) = (
                sum_r.checked_div(count),
                sum_g.checked_div(count),
                sum_b.checked_div(count),
            ) {
                dst[out_idx] = r as u8;
                dst[out_idx + 1] = g as u8;
                dst[out_idx + 2] = b as u8;
                dst[out_idx + 3] = src[out_idx + 3];
            } else {
                dst[out_idx..out_idx + BYTES_PER_PIXEL]
                    .copy_from_slice(&src[out_idx..out_idx + BYTES_PER_PIXEL]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur_pass_identity() {
        let src = vec![255, 0, 0, 255];
        let mut dst = vec![0; 4];

        blur_pass(&src, &mut dst, 1, 1, 1);

        assert_eq!(dst, src);
    }

    #[test]
    fn test_blur_pass_2x2_radius0() {
        let src = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ];

        let mut dst = vec![0; src.len()];

        blur_pass(&src, &mut dst, 2, 2, 0);

        assert_eq!(dst, src);
    }
}
