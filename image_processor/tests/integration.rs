use image_processor::process_image;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_mirror_plugin_roundtrip() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.png");
    let output = dir.path().join("output.png");
    let params = dir.path().join("params.json");

    // Определяем корень workspace (поднимаемся на один уровень из image_processor)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap();
    let plugin_dir = workspace_root.join("target/debug");

    let width = 2;
    let height = 2;
    let mut img = image::ImageBuffer::<image::Rgba<u8>, _>::new(width, height);
    img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
    img.put_pixel(0, 1, image::Rgba([0, 0, 255, 255]));
    img.put_pixel(1, 1, image::Rgba([255, 255, 0, 255]));
    img.save(&input).unwrap();

    fs::write(&params, r#"{"horizontal": true, "vertical": false}"#).unwrap();

    let res = process_image(&input, &output, "mirror_plugin", &params, &plugin_dir);
    assert!(res.is_ok(), "process_image failed: {:?}", res);

    let out_img = image::open(&output).unwrap().to_rgba8();
    assert_eq!(out_img.get_pixel(0, 0), &image::Rgba([0, 255, 0, 255]));
    assert_eq!(out_img.get_pixel(1, 0), &image::Rgba([255, 0, 0, 255]));
    assert_eq!(out_img.get_pixel(0, 1), &image::Rgba([255, 255, 0, 255]));
    assert_eq!(out_img.get_pixel(1, 1), &image::Rgba([0, 0, 255, 255]));
}

#[test]
fn test_nonexistent_input() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("noexist.png");
    let output = dir.path().join("out.png");
    let params = dir.path().join("params.json");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap();
    let plugin_dir = workspace_root.join("target/debug");

    fs::write(&params, "{}").unwrap();

    let res = process_image(&input, &output, "mirror_plugin", &params, &plugin_dir);
    assert!(res.is_err());
}
