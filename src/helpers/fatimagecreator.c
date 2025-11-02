#include <stdio.h>
#include <stdint.h>

typedef struct
{
    // FAT32 extended BPB is a comprised of several structures
    // contains ....
    //  BPB 2.0
    uint8_t jump_to_boot[3];        // Short jump + offset to bootloader code
    uint8_t oem[8];                 // OEM name string "MSDOS5.0"
    uint16_t bytes_per_sector;      // 0x200 > 512
    uint8_t sectors_per_cluster;    // 0x4 (cluster size 2024?)
    uint16_t reserved_sectors;      // 0x20 > 32
    uint8_t number_FATS;            // Number of FATs, usually2
    uint16_t root_entry_count;      // 0 for FAT32. Legacy root behaviour 12/16
    uint16_t total_logical_sectors; // 0 for FAT32/ Offset 0x020 changed if 0
    uint8_t media_descriptor;       // 0xF8 for fixed drives. Hard disk
    uint16_t sectors_per_FAT16;     // 0 for FAT32. uses 0x024

    // BPB 3.31
    uint16_t sectors_per_track;         // CHS Geometry 0x12 > 18 floppy. 0 for LBA, but not used.
                                        // 0x3f for fake CHS geometry
    uint16_t number_of_heads;           // 0x2 for floppy.
                                        // 0xff for fake CHS geometry hard disk. 255, because 256 crashed BIOSes
    uint32_t hidden_sectors;            // 0 non-partitioned. 0x3f on Ms specs example. AAP?
    uint32_t total_count_sectors_FAT32; // Total sectors for FAT32. 0x42a92 for 128MB example Ms

    // FAT32 Extended BPB
    uint32_t sectors_per_FAT32;       // Check sectors_per_FAT16; 0x214 Ms example for 128MB. Forbidden values!!
    uint16_t flags_FAT32;             // 0 Example Ms, but do some research.
    uint16_t version_FAT32;           // 0 Example Ms. Minor/major version number. Support for future version
    uint32_t root_cluster;            // Usually 2. First cluster of root directory
    uint16_t FSInfo_sector;           // Usually 1. Sector number of FSInfo structure
    uint16_t backup_boot_sector;      // Usually 6. Sector number of backup boot sector
    uint8_t reserved0[12];            // Reserved. Must be zero. Change only on formatting
    uint8_t physical_drive_number;    // 0x80 for hard disks. BIOS drive number
    uint8_t reserved1;                // Reserved. Must be zero for FAT32. Not clear purpose
    uint8_t boot_signature;           // 0x29 if next three fields are present
    uint32_t volume_ID;               // Volume serial number. Can be random I guess
    uint8_t volume_label[11];         // Volume label in ASCII. "TUSMUERTOS2" default
    uint8_t file_system_type[8];      // File system type label. "FAT32   "
    uint8_t reserved3[420];           // Padding to complete 512 bytes
    uint8_t boot_sector_signature[2]; // 0xAA55

} fat_descriptor;

int main(int argc, char **argv) // *argv[]
{

    printf("%s\n", argv[0]);
    printf("%s\n", argv[1]);
    printf("%s\n", argv[2]);
    printf("%s\n", argv[3]);

    return 0;
}
