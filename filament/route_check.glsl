// For the PIXEL part of a material that draws the light of one way through a flat-faced solid (route_light.mat,
// route_draw.mat): how much of this pixel's light to keep (0 to 1; a soft step one pixel wide at an edge).
// The pixel checks its own ray against the way named: through the face it came in by, then each face or film in turn,
// in the solid's own frame; plain arithmetic on the planes, no table.
// Needs the variables from (the point on the face the light came in by, index of refraction), way (the way, six bits
// per face or film met, in x and y; z: 1 if the light is only mirrored off the outside; w with sun.w: where the sun
// would put this point on the floor), sun (the sun's direction in the solid's frame), face (the outward normal of the
// face it came in by, and that face's place in the table), and the parameters planes, planeCount, films, filmCount,
// filmBand, strip, stripSoft.
float routeKeep() {
        // A pixel on the face, in the solid's own measure: the soft step at an edge is that wide. (How much room is left
        // jumps where a ray is no longer followed, so its own change from pixel to pixel would be no measure.)
        float pixel = length(dFdx(variable_from.xyz)) + length(dFdy(variable_from.xyz));
        float room = 1e3;
        {   // what is drawn is a box round the light: first of all the pixel has to be on the face the light came in by
            int entry = int(variable_face.w + 0.5);
            for (int j = 0; j < 40; j++) {
                if (j >= materialParams.planeCount) break;
                if (j == entry) continue;
                room = min(room, materialParams.planes[j].w - dot(materialParams.planes[j].xyz, variable_from.xyz));
            }
        }
        if (variable_way.z < 0.5 && room > -pixel) {
            // this pixel's own ray, along the way named: through the face, then each face or film in turn
            highp vec3 p = variable_from.xyz;
            float n = variable_from.w;
            highp vec3 d = refract(normalize(variable_sun.xyz), normalize(variable_face.xyz), 1.0 / n);
            int lo = int(variable_way.x + 0.5), hi = int(variable_way.y + 0.5);
            int on = -1;   // the film the ray stands on (it was just mirrored by it)
            for (int e = 0; e < 8; e++) {
                int id = (e < 4 ? (lo >> (6 * e)) : (hi >> (6 * (e - 4)))) & 63;
                if (id == 0) break;
                id -= 1;
                bool film = id >= 40;
                highp vec4 pl = film ? materialParams.films[id - 40] : materialParams.planes[id];
                highp float den = dot(pl.xyz, d);
                if (abs(den) < 1e-7) { room = -1.0; break; }
                highp float t = (pl.w - dot(pl.xyz, p)) / den;
                if (t < -1e-4) { room = -1.0; break; }
                // no film that mirrors this colour may lie between here and there
                for (int f = 0; f < 4; f++) {
                    if (f >= materialParams.filmCount) break;
                    if (f == on || (film && f == id - 40)) continue;
                    if (!(n > materialParams.filmBand[f].x && n < materialParams.filmBand[f].y)) continue;
                    highp vec4 fp = materialParams.films[f];
                    highp float s0 = fp.w - dot(fp.xyz, p), s1 = fp.w - dot(fp.xyz, p + d * t);
                    room = min(room, s0 * s1 > 0.0 ? min(abs(s0), abs(s1)) : -min(abs(s0), abs(s1)));
                }
                p += d * t;
                // and it has to get there inside the solid
                for (int j = 0; j < 40; j++) {
                    if (j >= materialParams.planeCount) break;
                    if (!film && j == id) continue;
                    room = min(room, materialParams.planes[j].w - dot(materialParams.planes[j].xyz, p));
                }
                if (room < -pixel) break;   // (this ray clearly does not go this way: no need to follow it further)
                d = reflect(d, pl.xyz);   // (at the last face the light leaves instead; where it then goes the walk has said)
                on = film ? id - 40 : -1;
            }
        }
        float keep = clamp(room / max(pixel, 1e-9) + 0.5, 0.0, 1.0);
        if (materialParams.strip.w > 0.0) {   // only what the band of sun falls on gets any
            float off = dot(materialParams.strip.xy, vec2(variable_way.w, variable_sun.w)) - materialParams.strip.z;
            if (materialParams.stripSoft.y > 0.0) off -= materialParams.stripSoft.y * floor(off / materialParams.stripSoft.y + 0.5);
            float band = materialParams.strip.w - abs(off);
            keep *= clamp(band / max(fwidth(band), 1e-9) + 0.5, 0.0, 1.0);
        }
        return keep;
}
