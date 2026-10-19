# Many things, each of its own kind, look and motion, in one picture (8 s, 1080x1920): solids, shapes, copies, films, words,
# each with effects of its own; the whole stage boiling with a fine grain. Written as edit calls.
out = ["say 0 background 120c22", "say 0 script ../../effects/fine_grain.wgsl", "set 0 js_fine_grain_amount 0.12"]
n = [0]
FX = "../../effects/"
def thing(kind, what, *lines):
    n[0] += 1
    out.append(f"add {kind} {what}")
    out.extend(l.replace("@", str(n[0])) for l in lines)
def fx(*names): return "say @ fx " + " ".join(FX + x + ".wgsl" for x in names)
def keys(name, *pairs): return [f"key @ {name} {t} {v}" for t, v in pairs]
# far back: a grainy glow, and a field of yellow dots waving
thing("shape", "circle", "say @ fill ff3fa4ff ff3fa400 ff3fa400", "set @ gradient radial", "set @ size 3.2", "set @ y 0.2", fx("grain"), "set @ js_grain_size 1.4")
thing("shape", "circle", "say @ fill ffd23f", "set @ size 0.05", "set @ copies 48", "set @ copy_columns 8", "set @ copy_x 0.22", "set @ copy_y 0.22", "set @ y -1.05",
      "set @ wave_y 0.06", "set @ wave_speed 1.2", "set @ stagger 1.5", fx("stuck_dots"))
# solids
thing("solid", "ring", "say @ fill ff6a3d", "set @ size 0.9", "set @ emissive 1.6", "set @ y 0.95", *keys("rotate_x", (0, 60), (8, 420)), fx("isf_god_rays"))
thing("solid", "cube", "say @ fill 3a7bff", "set @ size 0.55", "set @ x -0.55", "set @ y 0.25", *keys("rotate_y", (0, 0), (8, 270)), *keys("rotate_x", (0, 30), (8, -30)), fx("stuck_dots"))
thing("solid", "gem", "say @ fill glass", "set @ size 0.6", "set @ x 0.55", "set @ y 0.3", *keys("rotate_y", (0, 0), (8, -200)))
thing("solid", "ball", "say @ fill 9cff57", "set @ size 0.35", "set @ x 0.1", "set @ y -0.25", *keys("y", (0, -0.15), (2, -0.35), (4, -0.15), (6, -0.35), (8, -0.15)), fx("chroma"), "set @ js_chroma_amount 40")
thing("solid", "cube", "say @ fill 7a4dff", "set @ size 0.3", "set @ x -0.65", "set @ y -0.45", *keys("rotate", (0, 0), (8, 360)), fx("isf_noise_pixellate"))
# shapes
thing("shape", "star", "say @ fill ffe14a", "set @ size 0.28", "set @ points 5", "set @ x 0.66", "set @ y -0.35", "set @ copies 5", "set @ copy_rotate 72", "set @ copy_x 0.0",
      *keys("rotate", (0, 0), (8, 180)), fx("grain"), "say @ fill ffe14aff ff7b00ff ff7b0000", "set @ js_grain_size 1.3")
thing("shape", "cog", "say @ fill 34e0c8", "set @ size 0.42", "set @ points 12", "set @ x -0.62", "set @ y 1.05", *keys("rotate", (0, 0), (8, -240)), fx("technicolor"))
thing("shape", "spiral", "say @ fill none", "say @ stroke f6f1e7", "set @ stroke_width 6", "set @ size 0.5", "set @ x 0.62", "set @ y 1.15", *keys("rotate", (0, 0), (8, 300)), fx("isf_twirl"))
thing("shape", "rect", "say @ fill behind", "set @ round 0.25", "set @ size 0.55", "set @ wide 1.5", "set @ x 0.0", "set @ y 0.45", 
      *keys("x", (0, -0.25), (4, 0.25), (8, -0.25)), fx("isf_twirl"))
thing("shape", "arrow", "say @ fill ff4f6d", "set @ size 0.22", "set @ x -0.15", "set @ y -0.75", *keys("rotate", (0, -30), (4, 30), (8, -30)), fx("rgbshift"))
# films
thing("film", "../vid/happy.mp4", "set @ keyed 1", "set @ size 1.1", "set @ y -1.2", *keys("x", (0, -0.4), (4, 0.4), (8, -0.4)), fx("datamosh"))
thing("film", "../vid/popcat.mp4", "set @ keyed 1", "set @ size 0.55", "set @ x 0.6", "set @ y -0.95", fx("isf_rgb_trails_3_0"))
thing("film", "../vid/swinging.mp4", "set @ keyed 1", "set @ size 0.6", "set @ x -0.6", "set @ y -1.0", fx("isf_echo_trace"))
# words
thing("text", "../arial_black.ttf", "say @ text MOTOLII", "say @ fill f6f1e7", "set @ size 0.2", "set @ y 0.62", fx("chroma"), "set @ js_chroma_amount 70")
thing("text", "../arial_bold.ttf", "say @ text NEW VIDEO 10.19", "say @ fill ffd23f", "set @ size 0.07", "set @ y -0.6", fx("isf_dither_bayer"))
out += ["at 1.5", "still out/crowd_1.png", "at 5", "still out/crowd_5.png", "write out/show_crowd.mp4"]
print("\n".join(out))
