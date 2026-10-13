// (a probe: can SQLite itself work out how a value goes between two keys, from an expression kept in the document?
//  u: how far through the stretch (0 to 1), t: seconds into it, d: its length, a, b: the two keys' values, p1..p4: numbers said with the way)
#include <sqlite3.h>
#include <stdio.h>
#include <time.h>
static double ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec / 1e6; }
int main(void) {
    sqlite3* db; sqlite3_open(":memory:", &db);
    const char* ways[][2] = {
        {"linear", "a + (b - a) * u"},
        {"smooth", "a + (b - a) * u * u * (3 - 2 * u)"},
        {"hold", "case when u < 1 then a else b end"},
        {"wiggle", "a + (b - a) * u + p2 * sin(pi() * u) * (0.6 * sin(6.2832 * p1 * t) + 0.3 * sin(6.2832 * p1 * 2.3 * t + 1.7) + 0.1 * sin(6.2832 * p1 * 4.1 * t + 0.3))"},
        {"bounce", "a + (b - a) * (1 - abs(cos(pi() * 2.5 * u)) * pow(1 - u, 2))"}};
    for (int w = 0; w < 5; ++w) {
        char q[900]; sqlite3_stmt* s;
        snprintf(q, sizeof q, "with v(u, t, d, a, b, p1, p2, p3, p4) as (select ?, ?, ?, ?, ?, ?, ?, ?, ?) select (%s) from v", ways[w][1]);
        if (sqlite3_prepare_v2(db, q, -1, &s, 0) != SQLITE_OK) { printf("%-7s NOT taken: %s\n", ways[w][0], sqlite3_errmsg(db)); continue; }
        printf("%-7s", ways[w][0]);
        for (int i = 0; i <= 8; ++i) {
            const double u = i / 8.0, vals[] = {u, u * 2.0, 2.0, 1.0, 3.0, 3.0, 0.2, 0, 0};
            for (int k = 0; k < 9; ++k) sqlite3_bind_double(s, k + 1, vals[k]);
            sqlite3_step(s); printf(" %6.3f", sqlite3_column_double(s, 0)); sqlite3_reset(s);
        }
        const double t0 = ms();
        for (int i = 0; i < 200000; ++i) { for (int k = 0; k < 9; ++k) sqlite3_bind_double(s, k + 1, 0.37 + k); sqlite3_step(s); sqlite3_reset(s); }
        printf("   %.2f microseconds a reading\n", (ms() - t0) * 1000.0 / 200000.0);
        sqlite3_finalize(s);
    }
    return 0;
}
