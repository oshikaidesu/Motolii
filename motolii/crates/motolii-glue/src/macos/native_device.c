#include "wgpu.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef struct BNative {
    WGPUInstance instance;
    WGPUAdapter adapter;
    WGPUDevice device;
    WGPUQueue queue;
    WGPUTexture texture;
} BNative;

static void on_device(
    WGPURequestDeviceStatus status,
    WGPUDevice device,
    WGPUStringView message,
    void *userdata1,
    void *userdata2)
{
    (void)userdata2;
    if (status == WGPURequestDeviceStatus_Success) {
        *(WGPUDevice *)userdata1 = device;
        return;
    }
    fprintf(stderr, "wgpu device status %d: %s\n", (int)status, message.data ? message.data : "");
    *(WGPUDevice *)userdata1 = NULL;
}

BNative *b_native_open(uint32_t width, uint32_t height, char *error, size_t error_len)
{
    WGPUInstanceExtras extras;
    memset(&extras, 0, sizeof(extras));
    extras.chain.sType = WGPUSType_InstanceExtras;
    extras.backends = WGPUInstanceBackend_Metal;

    WGPUInstanceDescriptor instance_desc;
    memset(&instance_desc, 0, sizeof(instance_desc));
    instance_desc.nextInChain = &extras.chain;
    WGPUInstance instance = wgpuCreateInstance(&instance_desc);
    if (!instance) {
        snprintf(error, error_len, "wgpuCreateInstance failed");
        return NULL;
    }

    WGPUInstanceEnumerateAdapterOptions options;
    memset(&options, 0, sizeof(options));
    options.backends = WGPUInstanceBackend_Metal;
    WGPUAdapter adapters[4] = {0};
    size_t count = wgpuInstanceEnumerateAdapters(instance, &options, adapters);
    if (count == 0 || !adapters[0]) {
        snprintf(error, error_len, "no Metal adapter");
        wgpuInstanceRelease(instance);
        return NULL;
    }

    WGPUDevice device = NULL;
    WGPURequestDeviceCallbackInfo callback = WGPU_REQUEST_DEVICE_CALLBACK_INFO_INIT;
    callback.mode = WGPUCallbackMode_AllowProcessEvents;
    callback.callback = on_device;
    callback.userdata1 = &device;
    wgpuAdapterRequestDevice(adapters[0], NULL, callback);
    for (int i = 0; i < 2000 && !device; i++) {
        wgpuInstanceProcessEvents(instance);
        usleep(1000);
    }
    if (!device) {
        snprintf(error, error_len, "request device timed out");
        for (size_t i = 0; i < count && i < 4; i++) {
            if (adapters[i]) wgpuAdapterRelease(adapters[i]);
        }
        wgpuInstanceRelease(instance);
        return NULL;
    }
    for (size_t i = 1; i < count && i < 4; i++) {
        if (adapters[i]) wgpuAdapterRelease(adapters[i]);
    }

    WGPUTextureDescriptor texture_desc = WGPU_TEXTURE_DESCRIPTOR_INIT;
    texture_desc.usage = WGPUTextureUsage_CopySrc | WGPUTextureUsage_TextureBinding | WGPUTextureUsage_RenderAttachment;
    texture_desc.size = (WGPUExtent3D){width, height, 1};
    texture_desc.format = WGPUTextureFormat_BGRA8Unorm;
    texture_desc.mipLevelCount = 1;
    texture_desc.sampleCount = 1;
    WGPUTexture texture = wgpuDeviceCreateTexture(device, &texture_desc);
    if (!texture) {
        snprintf(error, error_len, "wgpuDeviceCreateTexture failed");
        wgpuDeviceRelease(device);
        wgpuAdapterRelease(adapters[0]);
        wgpuInstanceRelease(instance);
        return NULL;
    }

    BNative *native = calloc(1, sizeof(BNative));
    native->instance = instance;
    native->adapter = adapters[0];
    native->device = device;
    native->queue = wgpuDeviceGetQueue(device);
    native->texture = texture;
    return native;
}

void *b_native_instance(BNative *native) { return native->instance; }
void *b_native_device(BNative *native) { return native->device; }
void *b_native_texture(BNative *native) { return native->texture; }
void *b_native_metal_device(BNative *native) { return wgpuDeviceGetNativeMetalDevice(native->device); }
void *b_native_metal_queue(BNative *native) { return wgpuQueueGetNativeMetalCommandQueue(native->queue); }
