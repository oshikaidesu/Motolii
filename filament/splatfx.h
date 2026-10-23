// A PlayCanvas Gaussian splat shader effect (playcanvas/engine scripts/esm/gsplat: reveal-radial.mjs, reveal-rain.mjs,
// shader-effect-dissolve.mjs ...; MIT) as it is: its module run by QuickJS-ng with what it takes from PlayCanvas said
// small (Vec3, Color, its GSplatShaderEffect base: uniforms kept by name, the effect's time); its GLSL handed to the
// point cloud's look (points.h, points.mat), its attributes the knobs, its updateEffect asked each moment.
#pragma once
#include <functional>
#include <memory>
#include <string>
#include <utility>
#include <vector>
#include "script.h"

class SplatEffect {
public:
    explicit SplatEffect(const std::string& file);
    ~SplatEffect();
    bool ok() const;
    const std::string& trouble() const;
    const std::string& glsl() const;               // (its getShaderGLSL(): uniforms and the modifySplat... functions)
    const std::vector<ScriptKnob>& knobs() const;   // (its attributes: a number one knob, a Vec3 or Color three (_x _y _z, _r _g _b))
    // Its uniforms at `time` seconds into it: each knob as `value` says it (by its name; else as it comes), then its
    // updateEffect(time, dt).
    std::vector<std::pair<std::string, std::vector<float>>> uniforms(double time, double dt, const std::function<double(const std::string&, double)>& value);

    bool finished() const;
    // A knob's "as it comes" told again (an effect's aabbMin / aabbMax: the cloud's own box, so it works when just put on)
    void usual(const std::string& knob, double is);   // (it said so at the last moment: enabled = false, as PlayCanvas then takes its shader off)

private:
    struct Own;
    std::unique_ptr<Own> own;
};

// The point cloud's look (points.mat's text) with an effect's GLSL in it, built as matc builds it (Filament's own matp
// and filamat, for WebGPU and Metal): its uniforms made the look's parameters. The package's bytes, made once and kept
// in ~/Library/Caches/Motolii/splatfx (named by the two texts); empty, with `why`, if it could not be made.
namespace utils { class JobSystem; }
std::vector<uint8_t> splat_effect_look(const std::string& mat, const std::string& glsl, utils::JobSystem& jobs, std::string& why);   // (jobs: Filament's engine's own)
