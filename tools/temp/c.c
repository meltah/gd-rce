#include <stdio.h>

void asm_main();

void printn(int n) {
	printf("eax: %x\n", n);
}

int main() {
	printf("starting\n");
	asm_main();
}
