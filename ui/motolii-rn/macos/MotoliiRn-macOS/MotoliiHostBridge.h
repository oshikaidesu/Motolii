#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int64_t motolii_rn_host_create(
    const uint8_t *path,
    size_t path_len,
    uint64_t *out_host_handle,
    uint8_t *out,
    size_t out_cap);
int64_t motolii_rn_host_destroy(uint64_t host_handle, uint8_t *out, size_t out_cap);
int64_t motolii_rn_stage_register(
    uint64_t host_handle,
    uint64_t *out_stage_handle,
    uint8_t *out,
    size_t out_cap);
int64_t motolii_rn_stage_destroy(uint64_t stage_handle, uint8_t *out, size_t out_cap);
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
int64_t motolii_rn_stage_attach(
    uint64_t host_handle,
    uint64_t stage_handle,
    void *metal_layer,
    uint8_t *out,
    size_t out_cap);
int64_t motolii_rn_stage_resize_physical(
    uint64_t host_handle,
    uint64_t stage_handle,
    uint32_t width,
    uint32_t height,
    uint8_t *out,
    size_t out_cap);
int64_t motolii_rn_stage_draw(
    uint64_t host_handle,
    uint64_t stage_handle,
    uint8_t *out,
    size_t out_cap);
int64_t motolii_rn_stage_detach(
    uint64_t host_handle,
    uint64_t stage_handle,
    uint8_t *out,
    size_t out_cap);

#ifdef __cplusplus
}
#endif
