//! Библиотека для обработки изображений через FFI-плагины.

pub mod error;
pub mod plugin_loader;

use crate::error::Result;
use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::Path;

/// Загружает изображение, применяет плагин и сохраняет результат.
///
/// # Arguments
/// * `input_path` - путь к исходному PNG-файлу.
/// * `output_path` - путь для сохранения результата.
/// * `plugin_name` - имя плагина (без расширения).
/// * `params_path` - путь к текстовому файлу с параметрами (JSON).
/// * `plugin_dir` - директория, где находится библиотека плагина.
///
/// # Errors
/// Возвращает ошибку, если:
/// - входной файл не является PNG;
/// - файл параметров не существует или содержит невалидный JSON (ошибка передаётся плагину);
/// - плагин не загружен;
/// - выходную директорию нельзя создать или сохранить изображение.
pub fn process_image(
    input_path: &Path,
    output_path: &Path,
    plugin_name: &str,
    params_path: &Path,
    plugin_dir: &Path,
) -> Result<()> {
    // 1. Создаём выходную директорию, если её нет
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 2. Читаем изображение
    let img = image::open(input_path)?.to_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_raw();

    // 3. Читаем параметры
    let params = fs::read_to_string(params_path)?;

    // 4. Загружаем плагин
    let plugin = plugin_loader::Plugin::load(plugin_name, plugin_dir)?;

    // 5. Вызываем плагин
    plugin.process(width, height, &mut data, &params)?;

    // 6. Сохраняем результат
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).ok_or_else(|| {
        error::AppError::InvalidArgument("Failed to reconstruct image buffer".into())
    })?;
    buffer.save(output_path)?;

    Ok(())
}
