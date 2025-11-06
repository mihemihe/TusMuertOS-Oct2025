use anyhow::{Context, Ok, Result};
use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Size of the image in megabytes. At least 34 for FAT32
    #[arg(short, long)]
    size_mb: u64,

    /// Output image file path and name (can inlude relative folders)
    #[arg(short, long)]
    out: PathBuf,
}

fn main() -> Result<()> {
    let args: Args;
    let debug: bool = false;
    if debug {
        let synthetic_args = vec![
            "emptydisk01", // always include argv[0]
            "--size-mb",
            "5",
            "--out",
            "disk.img",
        ];
        args = Args::parse_from(synthetic_args);
        print!("{:?}\n", args);
    } else {
        args = Args::parse();
        print!("{:?}\n", args);
    }

    create_image(args.size_mb, &args.out)
}

fn create_image(size_mb: u64, out_path: &Path) -> Result<()> {
    let bytes = size_mb * 1024 * 1024;
    let aligned_bytes = (bytes / 512) * 512; // I do not really need this with 1024*1024 + mb, it is always divisible by 512

    let final_path = next_available_name(out_path)?;

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&final_path)
        .with_context(|| format!("Failed to create file: {}", final_path.display()))?;

    zero_fill(file, aligned_bytes)?;
    println!(
        "Created {} ({} bytes, aligned to 512-byte sectors)",
        final_path.display(),
        aligned_bytes
    );

    Ok(())
}

fn next_available_name(base: &Path) -> Result<PathBuf> {
    if !base.exists() {
        return Ok(base.to_path_buf());
    }

    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid filename")?;

    let extension = base.extension().and_then(|e| e.to_str()).unwrap_or("");

    for i in 1..=99 {
        let new_name = if extension.is_empty() {
            format!("{stem}{i:02}")
        } else {
            format!("{stem}{i:02}.{extension}")
        };

        let candidate = base.with_file_name(new_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("No available filename (tried up to 99 suffix)");
}

fn zero_fill(file: File, size: u64) -> Result<()> {
    let mut writer = BufWriter::new(file);
    const BUF_SIZE: usize = 1024 * 1024; // 1MB
    let zero_buff = [0u8; BUF_SIZE];

    let mut remaining = size;
    while remaining > 0 {
        let chunk = std::cmp::min(remaining, BUF_SIZE as u64);
        writer
            .write_all(&zero_buff[..chunk as usize])
            .context("Failed while writing zeros on the image")?;
        remaining -= chunk;
    }
    writer.flush().context("Failed flushing output")?;
    Ok(())
}
