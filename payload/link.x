SECTIONS {
	. = 0;
	.text 0 : {
		*(.pretext); *(.text);
	}
	.data : { *(.data*); }
	.rodata : { *(.rodata*); }
	.bss : { *(.bss); }
	.got.plt : { *(.got.plt) }
	.shstrtab : { *(.shstrtab) }
	/DISCARD/ : { *(.interp) }
}
