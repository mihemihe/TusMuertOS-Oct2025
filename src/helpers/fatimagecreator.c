#include <stdio.h>
#include <stdint.h>

#include <stdlib.h>
#include <string.h>

typedef struct
{
    // FAT32 extended BPB is a comprised of several structures
    // contains ....
    //  BPB 2.0
    uint8_t jump_to_boot[3];        // Short jump + offset to bootloader code
    uint8_t oem[8];                 // OEM name string "MSDOS5.0"
    uint16_t bytes_per_sector;      // 0x0200 > 512
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
    uint32_t hidden_sectors;            // 0 non-partitioned. 0x3f on Ms specs example. AAP? Cjheck again 0x3f
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

} __attribute__((packed)) FAT_bpb_descriptor; // 520 instead of 512 if not packed

typedef uint8_t *buffer_t;

int ReadSector(FILE *file, uint32_t number_sectors, uint32_t sector_size, buffer_t buf)
{
    const int FILL_ZERO = 0;
    // printf("Buffer address inside: %p\n", buf);
    int ret = fseek(file, 0, SEEK_SET);
    if (ret)
    {
        return ret;
    }
    fflush(stdout);

    memset(buf, FILL_ZERO, number_sectors * sector_size); // buffer location, value, bytes to fill
    ret = fread(buf, sector_size, number_sectors, file);  // buffer location, size element, number elements, file pointer.
    printf("%s\n", buf);
    fflush(stdout);
    return 0;
}

int inject_FAT_structure(FILE *output_file, FAT_bpb_descriptor *fat_bpb_desc)
{
    buffer_t buf512_boot = malloc(512); // FAT_bpb_descriptor buf512_boot = malloc(1)
    ReadSector(output_file, 1, 512, buf512_boot);

    FAT_bpb_descriptor *bpb_struct_ptr;                 // hint to the compiler about the size of the data pointed
    bpb_struct_ptr = (FAT_bpb_descriptor *)buf512_boot; // bpb_struct_ptr points to the buffer with FAT_bpb_descriptor structure pointer shape
    *fat_bpb_desc = *bpb_struct_ptr;                    // Dereference the pointer to get the structure data

    printf("\033[0;33mBytes per sector: %d\n", (*fat_bpb_desc).bytes_per_sector);

    return 0;
}

int main(int argc, char **argv) // *argv[]
{
    char *file_to_include = "document.txt";
    char *output_image = "fat32output.img";

    FILE *output_file = NULL;

    printf("FAT32 Image Creator\n");

    FAT_bpb_descriptor fat_bpb_desc = {0};
    printf("Size of FAT32 descriptor: %zu bytes\n", sizeof(FAT_bpb_descriptor));

    output_file = fopen(output_image, "r+");
    if (!output_file)
    {
        printf("Error opening file\n");
    }

    buffer_t buffer512 = malloc(512);

    int c = 33;
    // printf("Buffer address outside: %p\n", buffer512);
    printf("Buffer address int c: %p\n", &c);

    ReadSector(output_file, 1, 512, buffer512);
    printf("%s\n", buffer512);

    inject_FAT_structure(output_file, &fat_bpb_desc);

    return 0;
}
