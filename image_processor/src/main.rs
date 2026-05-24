use clap::Parser;
use image_processor::error::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input PNG image path
    input: PathBuf,

    /// Output PNG image path
    output: PathBuf,

    /// Plugin name (without extension)
    plugin: String,

    /// Path to parameters file
    params: PathBuf,

    /// Directory where plugin library resides (default: target/debug)
    #[arg(short, long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if !args.input.exists() {
        return Err(image_processor::error::AppError::InvalidArgument(format!(
            "Input file not found: {}",
            args.input.display()
        )));
    }
    if !args.params.exists() {
        return Err(image_processor::error::AppError::InvalidArgument(format!(
            "Params file not found: {}",
            args.params.display()
        )));
    }

    log::info!(
        "Processing {} -> {}",
        args.input.display(),
        args.output.display()
    );
    image_processor::process_image(
        &args.input,
        &args.output,
        &args.plugin,
        &args.params,
        &args.plugin_path,
    )?;

    log::info!("Image processed successfully!");
    Ok(())
}
