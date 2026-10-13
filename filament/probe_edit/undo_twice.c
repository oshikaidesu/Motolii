// (a probe: why undo fails after the same value is changed twice: doc.h's table and doc.h's way of setting a value)
#include <sqlite3.h>
#include <stdio.h>
static sqlite3* db;
static int conflict(void* c, int kind, sqlite3_changeset_iter* i) { (void)c; (void)i; printf("    conflict of kind %d (1 data, 2 not found, 3 conflict, 4 constraint, 5 foreign key)\n", kind); return SQLITE_CHANGESET_ABORT; }
typedef struct { int n; void* p; } Step;
static Step edit(const char* sql) {
    sqlite3_session* s; Step st = {0, 0};
    sqlite3session_create(db, "main", &s); sqlite3session_attach(s, "fact");
    sqlite3_exec(db, sql, 0, 0, 0);
    sqlite3session_changeset(s, &st.n, &st.p); sqlite3session_delete(s);
    sqlite3_changeset_iter* it; sqlite3changeset_start(&it, st.n, st.p);
    while (sqlite3changeset_next(it) == SQLITE_ROW) { const char* t; int cols, op, ind; sqlite3changeset_op(it, &t, &cols, &op, &ind); printf("    op %d (18 insert, 9 delete, 23 update)\n", op); }
    sqlite3changeset_finalize(it);
    return st;
}
static void undo(Step st) { int n; void* p; sqlite3changeset_invert(st.n, st.p, &n, &p); printf("  undo: %s\n", sqlite3changeset_apply(db, n, p, 0, conflict, 0) == SQLITE_OK ? "done" : "NOT done"); sqlite3_free(p); }
int main(int argc, char** argv) {
    sqlite3_open(":memory:", &db);
    sqlite3_exec(db, argv[1], 0, 0, 0);
    printf("%s\n", argv[1]);
    edit("insert into fact values(1, 'kind', 0, 'glass')");
    printf("  set x 0.1\n"); edit(argv[2]);
    printf("  set x 0.2\n"); Step b = edit(argv[3]);
    undo(b);
    return 0;
}
