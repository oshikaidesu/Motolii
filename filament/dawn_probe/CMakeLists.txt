# Built inside Filament's own build (both need the engine's private headers):
#   cmake -DMOTOLII_PROBE_DIR=<this folder> ...   (see filament_dawn_compute.patch, build.sh)
add_executable(dawn_probe dawn_probe.cpp)
target_link_libraries(dawn_probe PRIVATE filament)
target_include_directories(dawn_probe PRIVATE ${EXTERNAL}/stb)

# compute_door.h for programs built outside this build (glass_wgpu.sh)
add_library(motolii_door STATIC compute_door.cpp)
target_link_libraries(motolii_door PUBLIC filament)
