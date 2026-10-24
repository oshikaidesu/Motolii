// kit.h: what the glass pictures share (poster.mm, blocks.mm): a stage (Filament's engine, one view, a headless
// screen), a way to put a mesh on it, the round-edged glass block, and the picture of what is behind the blocks.
// Everything that draws is Filament as it is; glassbox.mat is the one material of ours that follows a sight line.
// In parts, read in this order (each a piece of the one before it, none alone):
#include "kit_stage.h"
#include "kit_things.h"
#include "kit_draw.h"
