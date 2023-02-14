/* Memory mapping for STM32F767ZI chip */
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 2M
  RAM   : ORIGIN = 0x20020000, LENGTH = 368K + 16K
  ITCM  : ORIGIN = 0x00000000, LENGTH = 16K /* Instruction Tighly Coupled Memory */
  DTCM  : ORIGIN = 0x20000000, LENGTH = 128K /* Data Tighly Coupled Memory */
}

SECTIONS
{
    .itcm : ALIGN(4)
    {
        *(.itcm .itcm.*);
        . = ALIGN(4);
    } > ITCM

    .dtcm : ALIGN(4)
    {
        *(.dtcm .dtcm.*);
        . = ALIGN(4);
    } > DTCM
}

/* You can then use something like this to place a variable into a specific section of memory:
 *  #[link_section = ".dtcm.BUFFER"]
 *  static mut BUF: [u8; 1024] = [3u8; 1024];
 *  Verifiable with: cargo size --release --example hello_world -- -A
 */

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */

/* Size of the heap (in bytes) */
/* _heap_size = 1024; */