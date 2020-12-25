#include <stdio.h>

int func(char buf[256]) {
	fprintf(stderr, "func: buf-size=%ju\n", sizeof buf);

	return 0;
}

int
main(void) {
	char xyz[1024];
	return func(xyz);
}
