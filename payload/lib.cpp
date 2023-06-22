#include "lib.hpp"

extern "C" void memcpy(void* dst, const void* src, size_t len) {
	char* d = (char*) dst;
	const char* s = (const char*) src;
	while(len) {
		*d++ = *s++;
		len--;
	}
}

size_t strlen(const char* str) {
	size_t len = 0;
	while (str[len]) {
		++len;
	}
	return len;
}

void print(const StringView& text) {
	WriteConsoleA(globals().stdout, text.data(), text.size(), nullptr, nullptr);
}

GameManager* getGameManager() {
	return from<GameManager*>(globals().gd, 0x3222d0);
}

StringView::StringView(const char* str) : m_data(str), m_size(strlen(str)) {}
StringView::StringView(const char* buf, size_t len) : m_data(buf), m_size(len) {}

const char* StringView::data() const { return m_data; }
size_t StringView::size() const { return m_size; }

float lerp(float a, float b, float p) {
	return a + (b - a) * p;
}
