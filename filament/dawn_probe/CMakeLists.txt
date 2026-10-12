# Built inside Filament's own build (it reaches the engine's driver commands, which are private headers):
#   cmake -DMOTOLII_PROBE_DIR=<this folder> ...   (see filament_dawn_compute.patch)
add_executable(dawn_probe dawn_probe.cpp)
target_link_libraries(dawn_probe PRIVATE filament)
target_include_directories(dawn_probe PRIVATE ${EXTERNAL}/stb)
