#include <vector>
#include <cstdio>
#include <cstdint>

// basic fibonacci seq. calculator using long ariphmetics by modulo 10^9 (to simplify base10 output) 

int main() {
    std::vector<uint64_t> a = { 1 }, b = { 0 }; // initial numbers

    constexpr uint64_t mod = 1'000'000'000;

    printf("fib 0 = 0\nfib 1 = 1\n");

    for (size_t i = 1; i < 1000; i++) {
        // (a, b) = (a+b, a)

        uint64_t carr = 0;
        size_t j = 0;
        while (j < b.size()) {
            uint64_t cur = carr + a[j] + b[j];
            carr = cur / mod;
            b[j] = a[j];
            a[j] = cur - carr * mod;
            j++;
        }
        while (j < a.size() && carr != 0) {
            uint64_t cur = carr + a[j];
            carr = cur / mod;
            b.push_back(a[j]);
            a[j] = cur - carr * mod;
            j++;
        }
        while (j < a.size()) {
            b.push_back(a[j++]);
        }
        if (carr != 0) {
            a.push_back(carr);
        }

        printf("fib %llu = %llu", i+1, a.back());
        for (size_t j = a.size()-2; j < a.size(); j--) {
            printf("%.09llu", a[j]);
        }
        putchar('\n');
    }
}