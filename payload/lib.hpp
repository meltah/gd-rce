#pragma once
#include "basic.hpp"
#include "gd.hpp"

class Hook {
public:
	uintptr_t addr;
	u8 orig[5];
	
	template <typename T>
	void init(uintptr_t addr, T dest, bool call=false);
	void restore();
};

template <size_t tail, typename F>
class THook {
public:
	uintptr_t addr;
	u8 trampoline[10 + tail];
	
	void init(uintptr_t addr, F dest);
	void restore();
	
	template <class... Args>
	auto operator()(Args... args);
};

template <typename T>
class Patch {
public:
	uintptr_t addr;
	T orig;

	void init(uintptr_t addr, T value);
	void restore();
};

struct SpeedRamp {
	float elapsed;
	float amountAdded;
};

struct AutoMovePath {
	float time;
	float x;
	float y;
	u8 xEaseIn;
	u8 xEaseOut;
	u8 yEaseIn;
	u8 yEaseOut;
};

struct AutoMove {
	float elapsed;
	float lastX;
	float lastY;
	int pathIndex;
	AutoMovePath path[16];
};

struct RewindMove {
	float x;
	float y;
	float elapsed;
};

struct RotateObjectLayer {
	float from, to;
	float elapsed;
};

struct WaitFor {
	float time;
	float x, y;
};

struct LevelState {
	float time;
	int firstGlitchTimes;
	SpeedRamp speedRamp;
	AutoMove autoMove;
	RewindMove rewindMove;
	RotateObjectLayer rotateObjectLayer;
	WaitFor waitFor;
};

struct CheckpointObjectExt : CheckpointObject {
	LevelState levelState;
};

struct Globals {
	Hook timeForXPosHook;
	Hook playAnimationCommmandHook;
	THook<1, void(__thiscall*)(PlayLayer*, bool)> playLayerDtorHook;
	THook<1, void(__thiscall*)(PlayLayer*, float)> updateHook;
	THook<1, void(__thiscall*)(PlayLayer*)> resetLevelHook;
	THook<1, CheckpointObject*(__thiscall*)(PlayLayer*)> createCheckpointHook;
	THook<1, void(__thiscall*)(PlayLayer*, CheckpointObject*)> loadFromCheckpointHook;
	Hook collectItemPatch;
	Patch<uint32_t> checkpointSizePatch;

#ifndef RELEASE
	bool shouldResetMusic;
#endif

	CCObject* updateObject;
	LevelState levelState;
	uintptr_t gd;
	void* stdout;
	void* fn_ptrs[0x1000];
};

Globals& globals();

#define DECLARE_FUNC(index, name, type) \
	template <class... Args> \
	ALWAYS_INLINE auto name(Args... args) { \
		return reinterpret_cast<type>(globals().fn_ptrs[index])(args...); \
	}

#define DEFINE_FUNC(index, value) globals().fn_ptrs[index] = reinterpret_cast<void*>(value)

DECLARE_FUNC(0x000, operator_new, void*(__cdecl*)(size_t))
DECLARE_FUNC(0x001, WriteConsoleA, int(__stdcall*)(void*, const char*, int, int*, void*))
DECLARE_FUNC(0x002, GetModuleHandleA, void*(__stdcall*)(const char*))
DECLARE_FUNC(0x003, VirtualProtect, int(__stdcall*)(void*, size_t, unsigned int, unsigned int*))
DECLARE_FUNC(0x004, FreeConsole, int(__stdcall*)())
DECLARE_FUNC(0x005, sprintf, void(*)(char*, const char*, ...))

DECLARE_FUNC(0x200, GJBaseGameLayer_collectItem, void(__thiscall*)(GJBaseGameLayer*, int, int))

DECLARE_FUNC(0x500, CCDirector_sharedDirector, CCDirector*(__cdecl*)())
DECLARE_FUNC(0x501, CCScene_create, CCScene*(__cdecl*)())
DECLARE_FUNC(0x502, CCLayer_CCLayer, CCLayer*(__thiscall*)(CCLayer*))
DECLARE_FUNC(0x503, CCLayer_init, bool(__thiscall*)(CCLayer*))
DECLARE_FUNC(0x504, CCObject_autorelease, void(__thiscall*)(CCObject*))
DECLARE_FUNC(0x505, CCDirector_replaceScene, void(__thiscall*)(CCDirector*, CCScene*))
DECLARE_FUNC(0x506, CCLabelBMFont_create, CCNode*(__cdecl*)(const char*, const char*))
DECLARE_FUNC(0x507, CCNode_addChild, void(__thiscall*)(CCNode*, CCNode*))
DECLARE_FUNC(0x508, CCScheduler_scheduleUpdateForTarget, void(__thiscall*)(CCScheduler*, CCObject*, int, bool));
DECLARE_FUNC(0x509, CCScheduler_unscheduleUpdateForTarget, void(__thiscall*)(CCScheduler*, CCObject*));
DECLARE_FUNC(0x50A, CCNode_setRotation, void(__thiscall*)(CCNode*, float));
DECLARE_FUNC(0x50B, CCDirector_getWinSize, CCSize(__thiscall*)(CCDirector*));

class StringView {
	const char* m_data;
	size_t m_size;
public:
	StringView(const char* str);
	StringView(const char* buf, size_t len);

	const char* data() const;
	size_t size() const;
};

template <typename T>
void Hook::init(uintptr_t addr, T dest, bool call) {
	this->addr = addr;
	__builtin_memcpy(&orig, (void*) addr, 5);
	unsigned int old, old2;
	VirtualProtect((void*) addr, 5, 0x40, &old);

	*(u8*)addr = call ? 0xE8 : 0xE9;
	*(int*)(addr + 1) = (uintptr_t)dest - addr - 5;

	VirtualProtect((void*) addr, 5, old, &old2);
}

inline void Hook::restore() {
	unsigned int old, old2;
	VirtualProtect((void*) addr, 5, 0x40, &old);

	__builtin_memcpy((void*) addr, orig, 5);

	VirtualProtect((void*) addr, 5, old, &old2);
}

template <size_t tail, typename F>
void THook<tail, F>::init(uintptr_t addr, F dest) {
	this->addr = addr;
	__builtin_memcpy(&trampoline, (void*) addr, tail + 5);
	if(trampoline[0] == 0xE8 || trampoline[0] == 0xE9) {
		// fix calls and jumps
		*(int*)(trampoline + 1) += addr - (uintptr_t)&trampoline;
	}
	trampoline[tail + 5] = 0xE9;
	*(int*)(trampoline + tail + 6) = addr + tail - (uintptr_t)trampoline;
	unsigned int old, old2;
	VirtualProtect((void*) addr, 5, 0x40, &old);

	*(u8*)addr = 0xE9;
	*(int*)(addr + 1) = (uintptr_t)dest - addr - 5;

	VirtualProtect((void*) addr, 5, old, &old2);
}

template <size_t tail, typename F>
void THook<tail, F>::restore() {
	if(trampoline[0] == 0xE8 || trampoline[0] == 0xE9) {
		// revert fix
		*(int*)(trampoline + 1) -= addr - (uintptr_t)&trampoline;
	}
	unsigned int old, old2;
	VirtualProtect((void*) addr, 5, 0x40, &old);

	__builtin_memcpy((void*) addr, &trampoline, 5);

	VirtualProtect((void*) addr, 5, old, &old2);
}

template <size_t tail, typename F> template <class... Args>
auto THook<tail, F>::operator()(Args... args) {
	return reinterpret_cast<F>(&trampoline)(args...);
}

template <typename T>
void Patch<T>::init(uintptr_t addr, T value) {
	this->addr = addr;

	unsigned int old, old2;
	VirtualProtect((void*) addr, 5, 0x40, &old);
	
	orig = *(T*)addr;
	*(T*)addr = value;
	
	VirtualProtect((void*) addr, 5, old, &old2);
}

template <typename T>
void Patch<T>::restore() {
	unsigned int old, old2;
	VirtualProtect((void*) addr, 5, 0x40, &old);
	
	*(T*)addr = orig;
	
	VirtualProtect((void*) addr, 5, old, &old2);
}

extern "C" void memcpy(void* dst, const void* src, size_t len);

size_t strlen(const char* str);

void print(const StringView& text);

template <class T, class F>
ALWAYS_INLINE T& from(F value, const intptr_t offset = 0) {
	return *reinterpret_cast<T*>(reinterpret_cast<uintptr_t>(value) + offset);
}

GameManager* getGameManager();

float lerp(float a, float b, float p);
