#include <cstdio>
#include <cstdlib>

#if 0
void fizzbuzz(int const x) {
    if (x % 15 == 0) {
        printf("FizzBuzz\n");
    } else if (x % 3 == 0) {
        printf("Fizz\n");
    } else if (x % 5 == 0) {
        printf("Buzz\n");
    } else {
        printf("%d\n", x);
    }
}
#endif

void fizzbuzz(int const x) {
    if (x % 5 == 0) {
        if (x % 3 == 0) {
            printf("FizzBuzz\n");
        } else {
            printf("Buzz\n");
        }
    } else if (x % 3 == 0) {
        printf("Fizz\n");
    } else {
        printf("%d\n", x);
    }
}