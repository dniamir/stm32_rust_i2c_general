MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K 
  /* RAM   : ORIGIN = 0x20000000, LENGTH = 32K */ 
  RAM   : ORIGIN = 0x20000000, LENGTH = 22K

  /* SRAM */
  SRAM1 :   ORIGIN = 0x20000000, LENGTH = 16K
  SRAM2 :   ORIGIN = 0x20004000, LENGTH = 6K
  CCMSRAM : ORIGIN = 0x20005800, LENGTH = 10K
}

SECTIONS {
  /*
  .sram1 : ALIGN(4) {
    *(.sram1 .sram1.*);
    . = ALIGN(4);
    } > SRAM1
  .sram2 : ALIGN(4) {
    *(.sram2 .sram2.*);
    . = ALIGN(4);
    } > SRAM2
  */
  .ccmsram : ALIGN(8) {
    *(.ccmsram .ccmsram.*);
    . = ALIGN(8);
    } > CCMSRAM
};