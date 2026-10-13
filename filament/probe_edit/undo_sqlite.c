// (a probe: is "save + undo" ready-made in SQLite's session extension? A document of two tables, three edits, each
//  edit's changeset kept; undo = the changeset inverted and applied, redo = the changeset applied.)
//   cc undo_sqlite.c -lsqlite3 -o ../out/undo_sqlite && ../out/undo_sqlite
#include <sqlite3.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static sqlite3* db;
static double ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec / 1e6; }
static int conflict(void* c, int e, sqlite3_changeset_iter* i) { (void)c; (void)e; (void)i; return SQLITE_CHANGESET_ABORT; }
static void show(const char* when) {
    sqlite3_stmt* q;
    printf("%-28s", when);
    sqlite3_prepare_v2(db, "select id, file, start, x from thing order by id", -1, &q, 0);
    while (sqlite3_step(q) == SQLITE_ROW) printf(" [%d %s start %.2f x %.2f]", sqlite3_column_int(q, 0), sqlite3_column_text(q, 1), sqlite3_column_double(q, 2), sqlite3_column_double(q, 3));
    sqlite3_finalize(q);
    sqlite3_prepare_v2(db, "select count(*) from key", -1, &q, 0);
    sqlite3_step(q); printf("  keys %d\n", sqlite3_column_int(q, 0)); sqlite3_finalize(q);
}
typedef struct { int n; void* p; } Step;
static Step edit(const char* sql) {   // one edit = one changeset
    sqlite3_session* s; Step st = {0, 0};
    sqlite3session_create(db, "main", &s); sqlite3session_attach(s, 0);
    if (sqlite3_exec(db, sql, 0, 0, 0) != SQLITE_OK) printf("edit failed: %s\n", sqlite3_errmsg(db));
    sqlite3session_changeset(s, &st.n, &st.p); sqlite3session_delete(s);
    return st;
}
static void undo(Step st) { int n; void* p; sqlite3changeset_invert(st.n, st.p, &n, &p); if (sqlite3changeset_apply(db, n, p, 0, conflict, 0) != SQLITE_OK) printf("undo failed\n"); sqlite3_free(p); }
static void redo(Step st) { if (sqlite3changeset_apply(db, st.n, st.p, 0, conflict, 0) != SQLITE_OK) printf("redo failed\n"); }
int main(void) {
    remove("../out/undo_probe.motolii");
    sqlite3_open("../out/undo_probe.motolii", &db);
    sqlite3_exec(db, "create table thing(id integer primary key, file text, start real, inpoint real, duration real, x real, y real, z real);"
                     "create table key(thing integer, value text, time real, is_ real, primary key(thing, value, time)) without rowid;", 0, 0, 0);
    Step a = edit("insert into thing values(1, 'happy.mp4', 0, 0, 6, 0, 0, 0)"); show("placed a film");
    Step b = edit("update thing set x = 0.5, start = 1.5 where id = 1"); show("moved it, later start");
    Step c = edit("insert into key values(1, 'x', 0, 0.5); insert into key values(1, 'x', 2, -0.5)"); show("two keys on x");
    printf("changeset sizes: %d, %d, %d bytes\n", a.n, b.n, c.n);
    undo(c); show("undo"); undo(b); show("undo"); undo(a); show("undo");
    redo(a); show("redo"); redo(b); show("redo"); redo(c); show("redo");
    // how dear: a thousand edits of a document of a thousand things, each undone
    sqlite3_exec(db, "begin", 0, 0, 0);
    for (int i = 2; i < 1000; ++i) { char q[200]; snprintf(q, sizeof q, "insert into thing values(%d, 'f.mp4', 0, 0, 6, 0, 0, 0)", i); sqlite3_exec(db, q, 0, 0, 0); }
    sqlite3_exec(db, "commit", 0, 0, 0);
    double t0 = ms();
    for (int i = 0; i < 1000; ++i) { char q[200]; snprintf(q, sizeof q, "update thing set x = %d where id = %d", i, 2 + i % 900); Step s = edit(q); undo(s); sqlite3_free(s.p); }
    printf("1000 edits each undone, on a file on disk: %.3f ms an edit+undo\n", (ms() - t0) / 1000.0);
    sqlite3_close(db);
    sqlite3_open("../out/undo_probe.motolii", &db); show("closed and opened again"); sqlite3_close(db);
    return 0;
}
