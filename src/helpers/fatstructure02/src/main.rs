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
#![allow(non_snake_case)]
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

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VBR {
    // FAT32 extended BPB is a comprised of several structures
    // contains ....
    //  BPB 2.0
    jump_to_boot: [u8; 3],      // Jump instruction to boot code
    oem: [u8; 8],               // OEM Name
    bytes_per_sector: u16,      // Bytes per sector
    sectors_per_cluster: u8,    // Sectors per cluster
    reserved_sectors: u16,      // Number of reserved sectors
    number_FATs: u8,            // Number of FATs
    root_entry_count: u16,      // Number of root directory entries (0 for FAT32)
    total_logical_sectors: u16, // Total sectors (if less than 65536)
    media_descriptor: u8,       // Media descriptor
    sectors_per_FAT16: u16,     // Sectors per FAT (if less than 65536)

    // BPB 3.31
    sectors_per_track: u16,         // Sectors per track (for BIOS)
    number_of_heads: u16,           // Number of heads (for BIOS)
    hidden_sectors: u32,            // Hidden sectors
    total_count_sectors_FAT32: u32, // Total sectors (if more than 65535)

    // FAT32 Extended BPB
    sectors_per_FAT32: u32,    // Sectors per FAT (FAT32 only)
    flags_FAT32: u16,          // Extended flags
    version_FAT32: u16,        // File system version
    root_cluster: u32,         // Root directory starting cluster
    FSInfo_sector: u16,        // FSInfo sector number
    backup_boot_sector: u16,   // Backup boot sector number
    reserved0: [u8; 12],       // Reserved
    physical_drive_number: u8, // Drive number
    reserved1: u8,             // Reserved
    boot_signature: u8,        // Boot signature
    volume_ID: u32,            // Volume ID
    volume_label: [u8; 11],    // Volume label
    file_system_type: [u8; 8], // File system type
    reserved3_for_code: [u8; 8],
    boot_sector_signature: [u8; 2], // Boot sector signature (0x55, 0xAA)
}

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
    println!("Validating file: {} and creating a FAT32 partition. Disk size 34Mb...", path_file.display());
    println!("...Partition starting at 1MiB(LBA 2048), size 33MiB (67584 sectors)");
    check_file_exist(&path_file)?;
    validate_file(&path_file)?;
    check_all_zeroes(&path_file)?;

    let partition_entry = F32_1stPartition {
        boot_flag: 0x80,
        start_chs: [0x00, 0x01, 0x10],
        part_type: 0x0C,
        end_chs: [0x03, 0xA0, 0x1F],
        start_lba: 2048u32,
        sectors: 67584u32,
    };
    display_partition_info(&partition_entry);
    inject_fat32_partition_table(&path_file, partition_entry)?; //Add as argument the size of the first partition, right now hardcoded to 34

    let vbr = VBR {
        // Common part
        jump_to_boot: [0xEB, 0x58, 0x90], // jmp short $+0x5A (0x58 + instruction size)
        //oem: *b"MSWIN4.1", //This is too fancy I guess
        //oem: {             // This is les fancy, more clear, but too verbose.
        //    let input = "MSDOS5.0";
        //    let mut oem = [0u8; 8];
        //   oem.copy_from_slice(input.as_bytes());
        //    oem
        //},
        oem: "TUSMUERT".as_bytes().try_into().unwrap(),
        //  BPB 2.0
        bytes_per_sector: 512,  // 0x00 0x02
        sectors_per_cluster: 1, //
        reserved_sectors: 32,   //0x20
        number_FATs: 2,
        root_entry_count: 0,      // Unused on FAT32 I think related to offset 0x42 (0x29/0x28)
        total_logical_sectors: 0, // FAT12/16, See the offset 0x20 for FAT32 and use 4 bytes
        media_descriptor: 0xF8,   // Fixed disk
        sectors_per_FAT16: 0,     // see offset 0x24 for FAT32

        // BPB 3.31
        sectors_per_track: 63,            // 0x3F, for interrupt 0x13!
        number_of_heads: 16,              // for interrupt 0x13
        hidden_sectors: 0,                // DOUBLE CHECK THIS VALUE, the USB is different 2048. Dont use to aling start of data
        total_count_sectors_FAT32: 67584, // somehow related to 2048 |0x20| 0x00 0x08 0x01 0x00 le, relevant for 0x13

        // FAT32 Extended BPB
        sectors_per_FAT32: 520, // |0x24| 0x08 0x02
        flags_FAT32: 0,
        version_FAT32: 0,
        root_cluster: 2,       // where the root directory cluster begins. The content of the dir is a file itself
        FSInfo_sector: 1,      // Sector 1
        backup_boot_sector: 6, // Sector 6
        reserved0: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        physical_drive_number: 0x80, // First HD. DL = 0x80 (Check if this is true in Bochs). Int 0x13 drive number
        reserved1: 0x00,
        boot_signature: 0x29,  // Extended BPB present |0x42|
        volume_ID: 1195118859, // 0B 11 3C 47 Microsoft suggest combining date and time
        volume_label: "TUSMUERTOS2".as_bytes().try_into().unwrap(),
        file_system_type: [0x46, 0x41, 0x54, 0x33, 0x32, 0x20, 0x20, 0x20], //FAT32bbb
        reserved3_for_code: [0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA], // this is dummy, calculate real value I think 420bytes
        boot_sector_signature: [0x55, 0xAA],
        // The FAT type is determined solely by the count of clusters on the volume
        // RootDirSectors = ((BPB_RootEntCnt * 32) + (BPB_BytsPerSec – 1)) / BPB_BytsPerSec . this gives 0, correct for FAT32
    };

    inject_VBR(&path_file, vbr)?;

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
    println!("File size: {} bytes", real_file_size);

    ensure!(real_file_size > 0, "File is empty! Zero size!");
    ensure!(real_file_size % (1024 * 1024) == 0, "File not multiple of 1024*1024 (1MB)");
    ensure!(real_file_size % 512 == 0, "File size is not neatly aligned to 512-bytes");
    ensure!(real_file_size >= 35651584, "File is too small for FAT32, ensure at least 34*1024*1024");
    println!("File is correct. Size is non-zero, multiple of 1MB, divisible by 512 bytes, and at least 34MB.");

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

fn inject_fat32_partition_table(pathfile: &PathBuf, partition_entry: F32_1stPartition) -> Result<()> {
    let mut bytes_to_inject = [0u8; 16];
    // using to_le_bytes() because integers are little-endian
    bytes_to_inject[0] = partition_entry.boot_flag;
    bytes_to_inject[1..4].copy_from_slice(&partition_entry.start_chs);
    bytes_to_inject[4] = partition_entry.part_type;
    bytes_to_inject[5..8].copy_from_slice(&partition_entry.end_chs);
    bytes_to_inject[8..12].copy_from_slice(&partition_entry.start_lba.to_le_bytes()); // I guess I need little endian
    bytes_to_inject[12..16].copy_from_slice(&partition_entry.sectors.to_le_bytes());

    println!("Partition entry (16 bytes): {:02X?}", bytes_to_inject);

    let mut file_handler = OpenOptions::new().read(true).write(true).open(pathfile)?;
    let offset_1st_partition_entry: u64 = 0x1BE; // byte 446 (off by one???)
    let offset_magic_boot_number: u64 = 0x1FE; // byte 510 (off by one???)

    file_handler.seek(SeekFrom::Start(offset_1st_partition_entry))?;
    file_handler.write_all(&bytes_to_inject)?;

    file_handler.seek(SeekFrom::Start(offset_magic_boot_number))?;
    file_handler.write_all(&[0x55, 0xAA])?;

    println!("Partition entry injected on MBR (16 bytes) offset {}", offset_1st_partition_entry);
    println!("Magic boot number injected on MBR (2 bytes) offset {}", offset_magic_boot_number);

    Ok(())
}

fn display_partition_info(p: &F32_1stPartition) {
    let bootable = if p.boot_flag == 0x80 { "Yes" } else { "No" };

    let part_type = match p.part_type {
        0x0B | 0x0C => "FAT32",
        0x0E => "FAT16 LBA",
        0x07 => "NTFS / exFAT",
        _ => "other value... (only recognizing FAT32 / FAT16 LBA / NTFS or exFAT)",
    };

    // Decode CHS values
    let (start_c, start_h, start_s) = chs_decode(p.start_chs);
    let (end_c, end_h, end_s) = chs_decode(p.end_chs);

    // Possible geometries to test, common ones are (255, 63) (4, 32)
    let geometries = [(255, 63), (128, 63), (64, 32), (16, 63), (8, 32), (4, 32)];

    let start_lba = p.start_lba;
    let sectors = p.sectors;
    // Try to find which geometry matches the Start LBA
    let mut detected = (255, 63);
    println!("\n[*] Detecting geometry that matches Start LBA = {}", start_lba);

    for (heads, sectors) in geometries {
        let lba = chs_to_lba(start_c, start_h, start_s, heads, sectors);

        println!("  → Testing geometry: Heads = {:3}, Sectors = {:2}  →  Computed LBA = {}", heads, sectors, lba);

        if lba == p.start_lba {
            println!("    ✓ Match found! Geometry matches Start LBA: Heads = {}, Sectors = {}", heads, sectors);
            detected = (heads, sectors);
            break;
        }
    }

    println!("[+] Final detected geometry: Heads = {}, Sectors = {}", detected.0, detected.1);

    let (heads_per_cylinder, sectors_per_track) = detected;

    let lba_calc = chs_to_lba(start_c, start_h, start_s, heads_per_cylinder, sectors_per_track);

    // Compute size
    let size_bytes = p.sectors as u64 * 512;
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    println!("Partition Info:");
    println!("  ├─ Bootable: {}", bootable);
    println!("  ├─ Type: {} (0x{:02X})", part_type, p.part_type);
    println!("  ├─ Start LBA, from field: {}", start_lba);
    println!("  ├─ Total sectors, from field: {}", sectors);
    println!("  ├─ Approx. size: {:.2} MiB. {}", size_mb, size_bytes);
    println!("  ├─ Geometry detected: {} heads * {} sectors", heads_per_cylinder, sectors_per_track);
    println!("  ├─ CHS range: {:02X?} → {:02X?}", p.start_chs, p.end_chs);
    println!("  ├─ Start CHS: C/H/S = {}/{}/{}   ({:02X?})", start_c, start_h, start_s, p.start_chs);
    println!("  └─ End CHS:   C/H/S = {}/{}/{}   ({:02X?})", end_c, end_h, end_s, p.end_chs);

    println!("  * CHS → LBA calculation:");
    println!("    ├─ Formula: (C * {} + H) * {} + (S - 1)", heads_per_cylinder, sectors_per_track);
    println!(
        "    ├─ Step 1: ({} * {} + {}) * {} + ({} - 1)",
        start_c, heads_per_cylinder, start_h, sectors_per_track, start_s
    );
    println!("    ├─ → LBA = {} (expected {})", lba_calc, start_lba);
    println!("    └─ → {:.2} MiB from disk start", (lba_calc as f64 * 512.0) / (1024.0 * 1024.0));
}

fn chs_to_lba(c: u16, h: u8, s: u16, heads_per_cylinder: u16, sectors_per_track: u16) -> u32 {
    ((c as u32 * heads_per_cylinder as u32 + h as u32) * sectors_per_track as u32 + (s as u32 - 1)) as u32
}

fn chs_decode(chs: [u8; 3]) -> (u16, u8, u16) {
    let head = chs[0];

    // Extract bits 0–5 for sector (mask 0b0011_1111)
    let sector = chs[1] & 0b0011_1111;

    // Extract bits 6–7 from byte 1 and bits 0–7 from byte 2 for cylinder
    let upper_cylinder_bits = ((chs[1] & 0b1100_0000) as u16) >> 6;
    let lower_cylinder_bits = chs[2] as u16;
    let cylinder = (upper_cylinder_bits << 8) | lower_cylinder_bits;

    (cylinder, head, sector.into())
}

fn inject_VBR(pathfile: &PathBuf, vbr: VBR) -> Result<()> {
    // Build the 512-byte VBR sector
    let mut bytes_to_inject = [0u8; 512];

    // DOS BPB + FAT32 EBPB layout
    bytes_to_inject[0x000..0x003].copy_from_slice(&vbr.jump_to_boot);
    bytes_to_inject[0x003..0x00B].copy_from_slice(&vbr.oem);
    bytes_to_inject[0x00B..0x00D].copy_from_slice(&vbr.bytes_per_sector.to_le_bytes());
    bytes_to_inject[0x00D] = vbr.sectors_per_cluster;
    bytes_to_inject[0x00E..0x010].copy_from_slice(&vbr.reserved_sectors.to_le_bytes());
    bytes_to_inject[0x010] = vbr.number_FATs;
    bytes_to_inject[0x011..0x013].copy_from_slice(&vbr.root_entry_count.to_le_bytes());
    bytes_to_inject[0x013..0x015].copy_from_slice(&vbr.total_logical_sectors.to_le_bytes());
    bytes_to_inject[0x015] = vbr.media_descriptor;
    bytes_to_inject[0x016..0x018].copy_from_slice(&vbr.sectors_per_FAT16.to_le_bytes());
    bytes_to_inject[0x018..0x01A].copy_from_slice(&vbr.sectors_per_track.to_le_bytes());
    bytes_to_inject[0x01A..0x01C].copy_from_slice(&vbr.number_of_heads.to_le_bytes());
    bytes_to_inject[0x01C..0x020].copy_from_slice(&vbr.hidden_sectors.to_le_bytes());
    bytes_to_inject[0x020..0x024].copy_from_slice(&vbr.total_count_sectors_FAT32.to_le_bytes());
    bytes_to_inject[0x024..0x028].copy_from_slice(&vbr.sectors_per_FAT32.to_le_bytes());
    bytes_to_inject[0x028..0x02A].copy_from_slice(&vbr.flags_FAT32.to_le_bytes());
    bytes_to_inject[0x02A..0x02C].copy_from_slice(&vbr.version_FAT32.to_le_bytes());
    bytes_to_inject[0x02C..0x030].copy_from_slice(&vbr.root_cluster.to_le_bytes());
    bytes_to_inject[0x030..0x032].copy_from_slice(&vbr.FSInfo_sector.to_le_bytes());
    bytes_to_inject[0x032..0x034].copy_from_slice(&vbr.backup_boot_sector.to_le_bytes());
    bytes_to_inject[0x034..0x040].copy_from_slice(&vbr.reserved0);
    bytes_to_inject[0x040] = vbr.physical_drive_number;
    bytes_to_inject[0x041] = vbr.reserved1;
    bytes_to_inject[0x042] = vbr.boot_signature;
    bytes_to_inject[0x043..0x047].copy_from_slice(&vbr.volume_ID.to_le_bytes());
    bytes_to_inject[0x047..0x052].copy_from_slice(&vbr.volume_label);
    bytes_to_inject[0x052..0x05A].copy_from_slice(&vbr.file_system_type);
    bytes_to_inject[0x05A..0x062].copy_from_slice(&vbr.reserved3_for_code);
    bytes_to_inject[0x1FE..0x200].copy_from_slice(&vbr.boot_sector_signature);

    // Write at offset 1 MiB (0x100000)
    let mut file_handler = OpenOptions::new().read(true).write(true).open(pathfile)?;
    let offset_vbr: u64 = 0x1_00_000;

    file_handler.seek(SeekFrom::Start(offset_vbr))?;
    file_handler.write_all(&bytes_to_inject)?;

    println!("FAT32 VBR injected (512 bytes) at offset {:#X} (LBA 2048)", offset_vbr);

    Ok(())
}
