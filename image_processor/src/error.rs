use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Image decode/encode error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Failed to load plugin: {0}")]
    PluginLoad(String),

    #[error("Failed to get symbol 'process_image': {0}")]
    PluginSymbol(#[from] libloading::Error),

    #[error("Parameter file parse error: {0}")]
    ParamParse(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
