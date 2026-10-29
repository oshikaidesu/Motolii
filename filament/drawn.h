// drawn.h: a picture ThorVG draws, put in the scene as a thing like any other. The scene and its things are
// Filament's (an entity with a place and a look); this only makes one whose look is ThorVG's picture: a flat sheet.
// The picture is a file: an After Effects film (Lottie) with all it holds, or an SVG. ThorVG draws it with WebGPU on
// Filament's own device straight into a texture Filament reads: nothing goes through the CPU, nothing is copied.
// Only on the WebGPU backend (BACKEND=webgpu, dawn_probe/kit_wgpu.sh), with ThorVG built against Dawn
// (dawn_probe/tvg_on_dawn.sh) and Filament patched to lend its device and take a texture (gpu_door.h).
// All of ThorVG's work is queued like Filament's own calls and runs on Filament's driver thread.
// In parts, read in this order (each a piece of the one before it, none alone):
#include "drawn_kept.h"
#include "drawn_made.h"
#include "absent_motion.h"   // (an open edge's ends as AbsentMotion draws them: drawn_flat.h)
#include "drawn_flat.h"
