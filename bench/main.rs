use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let size = 10_000_000; // 10MB worth of lines

    println!("Creating test file with {} lines...\n", size);
    create_test_file("input.txt", size)?;

    // === READING BENCHMARKS ===
    println!("=== READING BENCHMARKS ===");

    // Without buffer
    let start = Instant::now();
    read_without_buffer("input.txt")?;
    println!("Read without buffer: {:?}", start.elapsed());

    // With buffer
    let start = Instant::now();
    read_with_buffer("input.txt")?;
    println!("Read with buffer: {:?}", start.elapsed());

    println!();

    // === WRITING BENCHMARKS ===
    println!("=== WRITING BENCHMARKS ===");

    // Without buffer
    let start = Instant::now();
    write_without_buffer("output1.txt", size)?;
    println!("Write without buffer: {:?}", start.elapsed());

    // With buffer
    let start = Instant::now();
    write_with_buffer("output2.txt", size)?;
    println!("Write with buffer: {:?}", start.elapsed());

    // Cleanup
    std::fs::remove_file("input.txt")?;
    std::fs::remove_file("output1.txt")?;
    std::fs::remove_file("output2.txt")?;

    Ok(())
}

fn create_test_file(path: &str, lines: usize) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for i in 0..lines {
        writeln!(writer, "Line number {}: some test data here", i)?;
    }
    Ok(())
}

// Read byte-by-byte without buffer
fn read_without_buffer(path: &str) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 1];
    let mut count = 0;

    while file.read(&mut buffer)? > 0 {
        count += 1;
    }

    println!("  Bytes read: {}", count);
    Ok(())
}

// Read with buffer
fn read_with_buffer(path: &str) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let _ = line?;
        count += 1;
    }

    println!("  Lines read: {}", count);
    Ok(())
}

// Write line-by-line without buffer
fn write_without_buffer(path: &str, lines: usize) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    for i in 0..lines {
        writeln!(file, "Output line {}: processed data", i)?;
    }

    Ok(())
}

// Write with buffer
fn write_with_buffer(path: &str, lines: usize) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for i in 0..lines {
        writeln!(writer, "Output line {}: processed data", i)?;
    }

    Ok(())
}
