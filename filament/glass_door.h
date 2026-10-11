// The one part of the live glass that differs per OS: the buffers Filament draws into and the host shows without a copy,
// and the word that the GPU has finished one. macOS: glass_door_mac.mm. Windows: not written yet, see WINDOWS.md.
#pragma once

namespace filament {
class Engine;
class SwapChain;
}

constexpr int GLASS_DOOR_BUFFERS = 3;

// The engine, on the backend this OS's buffers need.
filament::Engine* glass_door_engine();
// Makes the buffers, w x h. finished(i) is called, on any thread, each time the GPU has finished a frame drawn into
// buffer i. To be called on the thread that draws: it also asks the OS to wake that thread on time. False: no buffers.
bool glass_door_open(filament::Engine* engine, int w, int h, void (*finished)(int));
// What Renderer::beginFrame takes to draw into buffer i.
filament::SwapChain* glass_door_chain(int i);
// Buffer i as the host takes it, or NULL if there is no such buffer. macOS: a CVPixelBufferRef, retained for the
// caller (who releases it).
void* glass_door_hand(int i);
