#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint64_t motolii_rn_host_create(const uint8_t *project_path_ptr, size_t project_path_len);
bool motolii_rn_host_destroy(uint64_t host_handle);
uint64_t motolii_rn_stage_register(uint64_t host_handle);
bool motolii_rn_stage_destroy(uint64_t stage_handle);
int64_t motolii_rn_host_read_snapshot_json(
    uint64_t host_handle,
    uint8_t *out,
    size_t out_cap);
int64_t motolii_rn_host_dispatch_intent_json(
    uint64_t host_handle,
    const uint8_t *intent_ptr,
    size_t intent_len,
    uint8_t *out,
    size_t out_cap);

#ifdef __cplusplus
}
#endif
