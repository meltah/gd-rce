BITS 32

global _asm_main
extern _printn

section .text
_asm_main:
	mov ecx, 0xFFFFFF80
	add cl, cl
	push ecx
	call _printn
	pop ecx
	ret
section .data
