#include <stdio.h>
#include <stdint.h>

typedef struct
{
    uint8_t jump_to_boot[3];        // Short jump + offset to bootloader code
    uint8_t oem[8];                 // OEM name string "MSDOS5.0"
    uint16_t bytes_per_sector;      // 0x200 > 512
    uint8_t sectors_per_cluster;    // 0x4 (cluster size 2024?)
    uint16_t reserved_sectors;      // 0x20 > 32
    uint8_t number_FATS;            // Number of FATs, usually2
    uint16_t root_entry_count;      // 0 for FAT32. Legacy root behaviour 12/16
    uint16_t total_logical_sectors; // 0 for FAT32/

} fat_descriptor;

int main(int argc, char **argv) // *argv[]
{

    printf("%s\n", argv[0]);
    printf("%s\n", argv[1]);
    printf("%s\n", argv[2]);
    printf("%s\n", argv[3]);

    return 0;
}
