/*
 * $ rm -v fizzbuzz *.o && g++-10 -ggdb -Ofast -c fizzbuzz.cxx && g++-10 -ggdb -Ofast -c main.cxx && g++-10 -ggdb -Ofast -o fizzbuzz main.o fizzbuzz.o
 */

#include <cstdlib>

extern void fizzbuzz(int const x);

int main(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        int const x = atoi(argv[i]);
        fizzbuzz(x);
    }
}