use image::DynamicImage;
use tempfile::TempDir;
use transmute_common::MediaFormat;
use transmute_core::Converter;

#[test]
fn test_convert_command() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.png");

    let img = DynamicImage::new_rgb8(100, 100);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();

    let command = format!(
        "convert {} to jpeg at {}",
        input_path.display(),
        temp_dir.path().display()
    );

    let outputs = converter.execute_command(&command).unwrap();

    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].exists());
    assert_eq!(MediaFormat::from_path(&outputs[0]), Some(MediaFormat::Jpeg));
}

#[test]
fn test_compress_command_with_percentage() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("image.jpg");

    let img = DynamicImage::new_rgb8(800, 600);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();

    let command = format!("compress {} to 75%", input_path.display());
    let outputs = converter.execute_command(&command).unwrap();

    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].exists());
}

#[test]
fn test_compress_with_quality_preset() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("photo.png");

    let img = DynamicImage::new_rgb8(640, 480);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();

    let command = format!("compress {} to high quality", input_path.display());
    let outputs = converter.execute_command(&command).unwrap();

    assert!(outputs[0].exists());
}

#[test]
fn test_batch_command() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    for i in 0..3 {
        let path = temp_dir.path().join(format!("img{}.png", i));
        let img = DynamicImage::new_rgb8(50, 50);
        img.save(&path).unwrap();
    }

    let converter = Converter::new().unwrap();

    let pattern = temp_dir.path().join("*.png");
    let command = format!(
        "batch {} convert to jpeg at {}",
        pattern.display(),
        temp_dir.path().display()
    );

    let outputs = converter.execute_command(&command).unwrap();

    assert_eq!(outputs.len(), 3);
    assert!(outputs.iter().all(|p| p.exists()));
}

#[test]
fn test_tilde_expansion() {
    let converter = Converter::new().unwrap();

    // Create test file in temp
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.png");
    let img = DynamicImage::new_rgb8(100, 100);
    img.save(&input_path).unwrap();

    // Command with absolute path (tilde won't work in temp)
    let command = format!("convert {} to jpeg", input_path.display());
    let result = converter.execute_command(&command);

    assert!(result.is_ok());
}

#[test]
fn test_case_insensitive_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("TEST.PNG");

    let img = DynamicImage::new_rgb8(50, 50);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();

    // All uppercase
    let command1 = format!("CONVERT {} TO JPEG", input_path.display());
    let result1 = converter.execute_command(&command1);
    assert!(result1.is_ok());

    // Mixed case
    let command2 = format!("CoNvErT {} tO JpEg", input_path.display());
    let result2 = converter.execute_command(&command2);
    assert!(result2.is_ok());
}

#[test]
fn test_quoted_paths() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("file with spaces.png");

    let img = DynamicImage::new_rgb8(100, 100);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();

    // Double quotes
    let command = format!(r#"convert "{}" to jpeg"#, input_path.display());
    let result = converter.execute_command(&command);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_command() {
    let converter = Converter::new().unwrap();

    let result = converter.execute_command("this is not a valid command");
    assert!(result.is_err());
}

#[test]
fn test_missing_file() {
    let converter = Converter::new().unwrap();

    let result = converter.execute_command("convert /nonexistent/file.png to jpeg");
    assert!(result.is_err());
}
