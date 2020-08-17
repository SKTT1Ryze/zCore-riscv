
OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    start = .;

    .text : {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : {
        PROVIDE_HIDDEN(kcounter_descriptor_begin = .);
        KEEP(*(.kcounter.descriptor))
        PROVIDE_HIDDEN(kcounter_descriptor_end = .);

        *(.data .data.*)
    }

    .stack : {
        *(.bss.stack)
    }

    .bss : {
        PROVIDE_HIDDEN(kcounters_arena_start = .);
	    KEEP(*(.kcounter.items))
	    PROVIDE_HIDDEN(kcounters_arena_end = .);

        *(.bss .bss.*)
    }

    PROVIDE(end = .);
}