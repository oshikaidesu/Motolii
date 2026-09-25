# naga's WGSL of filament_lib.frag -> a shelf module: drop the stub entry point, prefix every
# top-level name with fil_ so nothing collides with Motolii's prelude.
import re, sys
src = open(sys.argv[1]).read()
src = src.split("\nfn main_1()")[0]
src = re.sub(r"struct FragmentOutput \{[^}]*\}\n", "", src)
src = re.sub(r"var<private> [^\n]*\n", "", src)
names = re.findall(r"^fn (\w+)\(", src, re.M) + ["Refraction"]
for n in sorted(names, key=len, reverse=True):
    src = re.sub(rf"\b{n}\b", "fil_" + n, src)
hdr = ("// Generated: naga 29 GLSL frontend over motolii/reference/filament/filament_lib.frag (Filament shaders/src,\n"
       "// google/filament@41f996de8fcc2d6b60b73159aa1bc44a05a40700, Apache-2.0). Do not edit by hand.\n")
open(sys.argv[2], "w").write(hdr + src.strip() + "\n")
