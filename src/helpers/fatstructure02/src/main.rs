/*
// Use cli arguments to take the name of the file , mandatory, break if the name is not provided or is not found
// Examine the entire file and confirm is zeroed
// Validate the file size is multiple of 1024*1024
// Validate the file is at least 34MB
// If all validations work inject the FAT32 stucture,
//   MBR with magic number + partition table
//   FAT32 boot sector
//   FSInfo structure
//   Backup boot sector
//   FAT tables
//   Root directory structure
*/
//#![allow(warnings)] // REMOVE THIS !!!!!!!                 !
use anyhow::{Context, Ok, Result, bail, ensure};
use clap::Parser;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Path of the file to convert to FAT32
    #[arg(short, long)]
    filename: PathBuf,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct F32_1stPartition {
    boot_flag: u8,      // 0x80 = active/bootable, 0x00 = not active
    start_chs: [u8; 3], // CHS start (we keep zeroed — comment here in case I want CHS later)
    part_type: u8,      // partition type (0x0C = FAT32 LBA)
    end_chs: [u8; 3],   // CHS end (zeroed)
    start_lba: u32,     // little-endian
    sectors: u32,       // little-endian (number of 512-byte sectors)
} // 80 00 00 00 0C 00 00 00 00 08 00 00 00 08 01 00 (example to validate)

fn main() -> Result<()> {
    let args: Args;
    let debug: bool = true;
    if debug {
        let synthetic_args = vec![
            "emptydisk01", // always include argv[0]
            "--filename",
            //"../../../aaa.img",
            "../tempimage.img",
        ];
        args = Args::parse_from(synthetic_args);
        println!("{:?}", args);
    } else {
        args = Args::parse();
        println!("{:?}", args);
    }
    let path_file = args.filename;
    // Health checks, file exist, not zero, multiple of 512 and 1MB, and is all 0x00
    check_file_exist(&path_file)?;
    validate_file(&path_file)?;
    check_all_zeroes(&path_file)?;

    inject_fat32_partition_table(&path_file)?; //Add as argument the size of the first partition, right now hardcoded to 34

    Ok(())

    // Check if file exist first, if not chain errors
}

fn check_file_exist(pathfile: &Path) -> Result<()> {
    let file_exist = pathfile.is_file();

    if !file_exist {
        bail!("Could not find the file {}", pathfile.display())
    } else {
        println!("File found: {}", pathfile.display())
    }
    Ok(())
}

fn validate_file(pathfile: &PathBuf) -> Result<()> {
    let metadata =
        fs::metadata(pathfile).with_context(|| format!("Could not read the file {}. Interesting because the was there", pathfile.display()))?;
    let real_file_size = metadata.len();
    print!("File size: {} bytes", real_file_size);

    ensure!(real_file_size > 0, "File is empty! Zero size!");
    ensure!(real_file_size % (1024 * 1024) == 0, "File not multiple of 1024*1024 (1MB)");
    ensure!(real_file_size % 512 == 0, "File size is not neatly aligned to 512-bytes");
    ensure!(real_file_size >= 35651584, "File is too small for FAT32, ensure at least 34*1024*1024");

    Ok(())
}

fn check_all_zeroes(pathfile: &PathBuf) -> Result<()> {
    let file = fs::File::open(pathfile).with_context(|| format!("Could not open file {}", pathfile.display()))?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer).with_context(|| format!("Error reading {}", pathfile.display()))?;
        if bytes_read == 0 {
            break; // EOF
        }
        if buffer[..bytes_read].iter().any(|&b| b != 0) {
            bail!("File contains non-zero bytes!");
        }
    }

    println!("File is a greenfield, all 0x00!. We are ready to go.");
    Ok(())
}

fn inject_fat32_partition_table(pathfile: &PathBuf) -> Result<()> {
    let entry = F32_1stPartition {
        boot_flag: 0x80,
        start_chs: [0x00, 0x01, 0x10],
        part_type: 0x0C,
        end_chs: [0x03, 0xA0, 0x1F],
        start_lba: 2048u32,
        sectors: 67584u32,
    };

    let mut bytes_to_inject = [0u8; 16];
    bytes_to_inject[0] = entry.boot_flag;
    bytes_to_inject[1..4].copy_from_slice(&entry.start_chs);
    bytes_to_inject[4] = entry.part_type;
    bytes_to_inject[5..8].copy_from_slice(&entry.end_chs);
    bytes_to_inject[8..12].copy_from_slice(&entry.start_lba.to_le_bytes()); // I guess I need little endian
    bytes_to_inject[12..16].copy_from_slice(&entry.sectors.to_le_bytes());

    println!("Partition entry (16 bytes): {:02X?}", bytes_to_inject);

    let mut file_handler = OpenOptions::new().read(true).write(true).open(pathfile)?;
    let offset_1st_partition_entry: u64 = 0x1BE; // byte 446 (off by one???)
    let offset_magic_boot_number: u64 = 0x1FE; // byte 510 (off by one???)

    file_handler.seek(SeekFrom::Start(offset_1st_partition_entry))?;
    file_handler.write_all(&bytes_to_inject)?;

    file_handler.seek(SeekFrom::Start(offset_magic_boot_number))?;
    file_handler.write_all(&[0x55, 0xAA])?;

    Ok(())
}
