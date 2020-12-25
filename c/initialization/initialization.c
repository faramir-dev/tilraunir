#include <stdio.h>

static const int x[] = {[0 ... 16] = -1, -2, -3};
static const int y[17] = {1, 2, 3};

int
main(void) {
	printf("x=");
	for (size_t i = 0; i < sizeof(x) / sizeof(*x); ++i)
		printf("%s%d", i ? ", " : "", x[i]);
	printf("\n");

	printf("y=");
	for (size_t i = 0; i < sizeof(y) / sizeof(*y); ++i)
		printf("%s%d", i ? ", " : "", y[i]);
	printf("\n");

	return 0;
}
