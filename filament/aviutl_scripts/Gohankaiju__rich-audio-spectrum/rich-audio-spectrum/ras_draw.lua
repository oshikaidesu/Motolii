-- ras_draw.lua : rich-audio-spectrum 描画中核 (geometry / render / style)
-- 頂点色はストレートα (Phase 0実測、仕様書8.5節)。
-- 座標系: tempbuffer中心原点 (Phase 0実測)。render_to_object() が
-- tempbufferへ一括描画してオブジェクト画像に戻す (仕様書8.2節ルートA)。

local draw = {}
local core = require("ras_core")

local max, min, sqrt, abs = math.max, math.min, math.sqrt, math.abs

-- ============================================================
-- style
-- ============================================================

-- 0xRRGGBB → {r,g,b} (0..1)
function draw.rgb(col)
    col = col or 0xffffff
    local r = math.floor(col / 65536) % 256
    local g = math.floor(col / 256) % 256
    local b = col % 256
    return { r / 255, g / 255, b / 255 }
end

-- c1,c2: {r,g,b}, t: 0..1
function draw.grad(c1, c2, t)
    t = core.clamp(t, 0, 1)
    return c1[1] + (c2[1] - c1[1]) * t,
           c1[2] + (c2[2] - c1[2]) * t,
           c1[3] + (c2[3] - c1[3]) * t
end

-- ============================================================
-- geometry (obj非依存: 頂点リストを返すだけ)
-- ============================================================

-- 四角形を頂点リスト(4頂点/面, {x,y,z,r,g,b,a})へ追加。
-- r2等を指定すると上端(y0側)の色を変えられる(縦グラデーション)。
function draw.push_quad(v, x0, y0, x1, y1, r, g, b, a, r2, g2, b2, a2)
    r2 = r2 or r; g2 = g2 or g; b2 = b2 or b; a2 = a2 or a
    local i = #v
    v[i + 1] = { x0, y0, 0, r2, g2, b2, a2 }
    v[i + 2] = { x1, y0, 0, r2, g2, b2, a2 }
    v[i + 3] = { x1, y1, 0, r, g, b, a }
    v[i + 4] = { x0, y1, 0, r, g, b, a }
end

-- 全頂点のyを反転した複製を追加(上下ミラー用)
function draw.append_mirrored(v)
    local n = #v
    for i = 1, n, 4 do
        -- 面単位で頂点順を保つ(時計回り維持のため0,1,2,3 → 3,2,1,0で反転)
        local p0, p1, p2, p3 = v[i], v[i + 1], v[i + 2], v[i + 3]
        v[#v + 1] = { p3[1], -p3[2], p3[3], p3[4], p3[5], p3[6], p3[7] }
        v[#v + 1] = { p2[1], -p2[2], p2[3], p2[4], p2[5], p2[6], p2[7] }
        v[#v + 1] = { p1[1], -p1[2], p1[3], p1[4], p1[5], p1[6], p1[7] }
        v[#v + 1] = { p0[1], -p0[2], p0[3], p0[4], p0[5], p0[6], p0[7] }
    end
    return v
end

-- スペクトラムバー。bands: 0..1配列。
-- geo: { w, h, bar_ratio(0..1), mirror, col1={r,g,b}, col2, grad_by_amp, alpha }
function draw.bars(bands, geo)
    local n = #bands
    local v = {}
    if n < 1 then return v end
    local W, H = geo.w, geo.h
    local slot = W / n
    -- 太さを整数pxに丸め、左端を整数座標へスナップして全バーを同一幅にする
    -- (スロット幅が割り切れない場合の±1px揺れはバー間隔側に吸収させる)
    local bw = max(1, math.floor(slot * core.clamp(geo.bar_ratio or 0.7, 0.05, 1) + 0.5))
    for i = 1, n do
        local val = core.clamp(bands[i] or 0, 0, 1)
        if val > 0.003 then
            local cx = -W / 2 + (i - 0.5) * slot
            local x0 = math.floor(cx - bw / 2 + 0.5)
            local x1 = x0 + bw
            local t = geo.grad_by_amp and val or ((i - 1) / max(n - 1, 1))
            local r, g, b = draw.grad(geo.col1, geo.col2, t)
            local hgt = val * H
            if geo.mirror then
                draw.push_quad(v, x0, -hgt / 2, x1, hgt / 2, r, g, b, geo.alpha)
            else
                draw.push_quad(v, x0, H / 2 - hgt, x1, H / 2, r, g, b, geo.alpha)
            end
        end
    end
    return v
end

-- 中央基線のバー (DTMer互換: 基線から上へ、ミラーONで下側も描く)。
-- geo: { w, hpx(表示値1.0のpx), bw(バー太さpx、スロット幅+1でクランプ),
--        mirror, col1, col2, grad_by_amp, alpha }
function draw.bars_center(bands, geo)
    local n = #bands
    local v = {}
    if n < 1 then return v end
    local W = geo.w
    local slot = W / n
    -- 太さを整数pxに丸め、左端を整数座標へスナップして全バーを同一幅にする
    local bw = max(1, math.floor(min(geo.bw or 5, slot + 1) + 0.5))
    for i = 1, n do
        local val = core.clamp(bands[i] or 0, 0, 1)
        if val > 0.002 then
            local cx = -W / 2 + (i - 0.5) * slot
            local x0 = math.floor(cx - bw / 2 + 0.5)
            local x1 = x0 + bw
            local t = geo.grad_by_amp and val or ((i - 1) / max(n - 1, 1))
            local r, g, b = draw.grad(geo.col1, geo.col2, t)
            local h = val * geo.hpx
            draw.push_quad(v, x0, -h, x1, 0, r, g, b, geo.alpha)
            if geo.mirror then
                draw.push_quad(v, x0, 0, x1, h, r, g, b, geo.alpha)
            end
        end
    end
    return v
end

-- 点列。bands: 0..1配列。geo追加: dot_size
function draw.dots(bands, geo)
    local n = #bands
    local v = {}
    if n < 1 then return v end
    local W, H = geo.w, geo.h
    local slot = W / n
    local r0 = max((geo.dot_size or 6) / 2, 1)
    for i = 1, n do
        local val = core.clamp(bands[i] or 0, 0, 1)
        local cx = -W / 2 + (i - 0.5) * slot
        local y
        if geo.mirror then
            y = -val * H / 2
        else
            y = H / 2 - val * H
        end
        local t = geo.grad_by_amp and val or ((i - 1) / max(n - 1, 1))
        local r, g, b = draw.grad(geo.col1, geo.col2, t)
        draw.push_quad(v, cx - r0, y - r0, cx + r0, y + r0, r, g, b, geo.alpha)
    end
    return v
end

-- 折れ線をリボン(太さ付き)で描く。
-- points: 値配列。signed=false: 0..1をベースラインから上へ / true: -1..1を中央基準。
-- geo: { w, h, thickness, mirror, signed, col1, col2, alpha }
function draw.line(points, geo)
    local m = #points
    local v = {}
    if m < 2 then return v end
    local W, H = geo.w, geo.h
    local th = max(geo.thickness or 3, 0.5)
    local xs, ys = {}, {}
    for i = 1, m do
        xs[i] = -W / 2 + (i - 1) / (m - 1) * W
        local val = points[i] or 0
        if geo.signed then
            ys[i] = -core.clamp(val, -1, 1) * H / 2
        elseif geo.mirror then
            ys[i] = -core.clamp(val, 0, 1) * H / 2
        else
            ys[i] = H / 2 - core.clamp(val, 0, 1) * H
        end
    end
    for i = 1, m - 1 do
        local dx, dy = xs[i + 1] - xs[i], ys[i + 1] - ys[i]
        local len = sqrt(dx * dx + dy * dy)
        if len > 0.0001 then
            local nx, ny = -dy / len * th / 2, dx / len * th / 2
            local t = (i - 1) / max(m - 2, 1)
            local r, g, b = draw.grad(geo.col1, geo.col2, t)
            local k = #v
            v[k + 1] = { xs[i] + nx, ys[i] + ny, 0, r, g, b, geo.alpha }
            v[k + 2] = { xs[i + 1] + nx, ys[i + 1] + ny, 0, r, g, b, geo.alpha }
            v[k + 3] = { xs[i + 1] - nx, ys[i + 1] - ny, 0, r, g, b, geo.alpha }
            v[k + 4] = { xs[i] - nx, ys[i] - ny, 0, r, g, b, geo.alpha }
        end
    end
    return v
end

-- 塗り波形 (上下対称)。samples: -1..1配列(絶対値で塗る)。
function draw.wave_fill(samples, geo)
    local m = #samples
    local v = {}
    if m < 2 then return v end
    local W, H = geo.w, geo.h
    for i = 1, m - 1 do
        local x0 = -W / 2 + (i - 1) / (m - 1) * W
        local x1 = -W / 2 + i / (m - 1) * W
        local a0 = core.clamp(abs(samples[i] or 0), 0, 1) * H / 2
        local a1 = core.clamp(abs(samples[i + 1] or 0), 0, 1) * H / 2
        a0 = max(a0, 0.5)
        a1 = max(a1, 0.5)
        local t = (i - 1) / max(m - 2, 1)
        local r, g, b = draw.grad(geo.col1, geo.col2, t)
        local k = #v
        v[k + 1] = { x0, -a0, 0, r, g, b, geo.alpha }
        v[k + 2] = { x1, -a1, 0, r, g, b, geo.alpha }
        v[k + 3] = { x1, a1, 0, r, g, b, geo.alpha }
        v[k + 4] = { x0, a0, 0, r, g, b, geo.alpha }
    end
    return v
end

-- ============================================================
-- radial geometry (仕様書6.4節)
-- ============================================================

-- 円周上のバー。bands: 0..1配列。
-- geo: { radius, barlen, thickness, a0_deg, a1_deg, rot_deg,
--        dir(0=out,1=in,2=both), inner_min, col1, col2, grad_by_amp, alpha }
function draw.ring_bars(bands, geo)
    local n = #bands
    local v = {}
    if n < 1 then return v end
    local R = geo.radius
    local len = geo.barlen
    local th = max(geo.thickness or 4, 0.5) / 2
    local a0 = math.rad(geo.a0_deg or 0)
    local a1 = math.rad(geo.a1_deg or 360)
    local rot = math.rad(geo.rot_deg or 0)
    local inner_min = max(geo.inner_min or 2, 0)
    for i = 1, n do
        local val = core.clamp(bands[i] or 0, 0, 1)
        if val > 0.003 then
            local ang = a0 + (a1 - a0) * (i - 0.5) / n + rot - math.pi / 2
            local ux, uy = math.cos(ang), math.sin(ang)
            local tx, ty = -uy * th, ux * th   -- 接線方向(バー幅)
            local r_base, r_tip
            if geo.dir == 1 then
                r_base = R
                r_tip = max(R - val * len, inner_min)   -- 中心跨ぎclamp (仕様6.4節)
            elseif geo.dir == 2 then
                r_base = max(R - val * len * 0.5, inner_min)
                r_tip = R + val * len * 0.5
            else
                r_base = R
                r_tip = R + val * len
            end
            local t
            if geo.grad_by_amp then
                t = val
            else
                t = (i - 1) / max(n - 1, 1)
                if geo.grad_mirror then t = 1 - math.abs(2 * t - 1) end
            end
            local r, g, b = draw.grad(geo.col1, geo.col2, t)
            local k = #v
            v[k + 1] = { ux * r_base + tx, uy * r_base + ty, 0, r, g, b, geo.alpha }
            v[k + 2] = { ux * r_tip + tx, uy * r_tip + ty, 0, r, g, b, geo.alpha }
            v[k + 3] = { ux * r_tip - tx, uy * r_tip - ty, 0, r, g, b, geo.alpha }
            v[k + 4] = { ux * r_base - tx, uy * r_base - ty, 0, r, g, b, geo.alpha }
        end
    end
    return v
end

-- 円周上のスプラインライン(閉ループ)。points: 0..1配列(再サンプル済み)。
function draw.ring_line(points, geo)
    local m = #points
    local v = {}
    if m < 3 then return v end
    local R = geo.radius
    local len = geo.barlen
    local th = max(geo.thickness or 3, 0.5) / 2
    local a0 = math.rad(geo.a0_deg or 0)
    local a1 = math.rad(geo.a1_deg or 360)
    local rot = math.rad(geo.rot_deg or 0)
    local full = math.abs((geo.a1_deg or 360) - (geo.a0_deg or 0)) >= 359.9
    local xs, ys = {}, {}
    for i = 1, m do
        local frac = full and ((i - 1) / m) or ((i - 1) / (m - 1))
        local ang = a0 + (a1 - a0) * frac + rot - math.pi / 2
        local val = core.clamp(points[i] or 0, 0, 1)
        local r
        if geo.dir == 1 then
            r = max(R - val * len, max(geo.inner_min or 2, 0))
        else
            r = R + val * len
        end
        xs[i] = math.cos(ang) * r
        ys[i] = math.sin(ang) * r
    end
    local segs = full and m or (m - 1)
    for i = 1, segs do
        local j = (i % m) + 1
        local dx, dy = xs[j] - xs[i], ys[j] - ys[i]
        local d = sqrt(dx * dx + dy * dy)
        if d > 0.0001 then
            local nx, ny = -dy / d * th, dx / d * th
            local t = (i - 1) / max(segs - 1, 1)
            if geo.grad_mirror then t = 1 - math.abs(2 * t - 1) end
            local r, g, b = draw.grad(geo.col1, geo.col2, t)
            local k = #v
            v[k + 1] = { xs[i] + nx, ys[i] + ny, 0, r, g, b, geo.alpha }
            v[k + 2] = { xs[j] + nx, ys[j] + ny, 0, r, g, b, geo.alpha }
            v[k + 3] = { xs[j] - nx, ys[j] - ny, 0, r, g, b, geo.alpha }
            v[k + 4] = { xs[i] - nx, ys[i] - ny, 0, r, g, b, geo.alpha }
        end
    end
    return v
end

-- 円周上の点列。
function draw.ring_dots(bands, geo)
    local n = #bands
    local v = {}
    if n < 1 then return v end
    local R = geo.radius
    local len = geo.barlen
    local rr = max((geo.dot_size or 6) / 2, 1)
    local a0 = math.rad(geo.a0_deg or 0)
    local a1 = math.rad(geo.a1_deg or 360)
    local rot = math.rad(geo.rot_deg or 0)
    for i = 1, n do
        local val = core.clamp(bands[i] or 0, 0, 1)
        local ang = a0 + (a1 - a0) * (i - 0.5) / n + rot - math.pi / 2
        local r
        if geo.dir == 1 then
            r = max(R - val * len, max(geo.inner_min or 2, 0))
        else
            r = R + val * len
        end
        local cx, cy = math.cos(ang) * r, math.sin(ang) * r
        local t
        if geo.grad_by_amp then
            t = val
        else
            t = (i - 1) / max(n - 1, 1)
            if geo.grad_mirror then t = 1 - math.abs(2 * t - 1) end
        end
        local cr, cg, cb = draw.grad(geo.col1, geo.col2, t)
        draw.push_quad(v, cx - rr, cy - rr, cx + rr, cy + rr, cr, cg, cb, geo.alpha)
    end
    return v
end

-- ============================================================
-- stereo scope (仕様書6.6節)
-- ============================================================

-- リサージュ変換: L/R → 45度回転座標 (mono=縦線)
-- L,R: -1..1配列。points: 使用点数。scale: 半幅px。
function draw.lissajous_points(L, R, n, points, scale)
    local xs, ys = {}, {}
    local m = math.min(points, n)
    local inv = 1 / math.sqrt(2)
    for i = 1, m do
        local si = math.floor((i - 1) / max(m - 1, 1) * (n - 1)) + 1
        local l = core.clamp(L[si] or 0, -1, 1)
        local r = core.clamp(R[si] or 0, -1, 1)
        xs[i] = (l - r) * inv * scale
        ys[i] = -(l + r) * inv * scale
    end
    return xs, ys, m
end

-- 点列をリボン折れ線で結ぶ (開ループ、xy直接指定)
function draw.polyline(xs, ys, m, geo)
    local v = {}
    local th = max(geo.thickness or 2, 0.5) / 2
    for i = 1, m - 1 do
        local dx, dy = xs[i + 1] - xs[i], ys[i + 1] - ys[i]
        local d = sqrt(dx * dx + dy * dy)
        if d > 0.0001 then
            local nx, ny = -dy / d * th, dx / d * th
            local t = (i - 1) / max(m - 2, 1)
            local r, g, b = draw.grad(geo.col1, geo.col2, t)
            local k = #v
            v[k + 1] = { xs[i] + nx, ys[i] + ny, 0, r, g, b, geo.alpha }
            v[k + 2] = { xs[i + 1] + nx, ys[i + 1] + ny, 0, r, g, b, geo.alpha }
            v[k + 3] = { xs[i + 1] - nx, ys[i + 1] - ny, 0, r, g, b, geo.alpha }
            v[k + 4] = { xs[i] - nx, ys[i] - ny, 0, r, g, b, geo.alpha }
        end
    end
    return v
end

-- xy点列を小さな四角ドットに
function draw.xy_dots(xs, ys, m, geo)
    local v = {}
    local rr = max((geo.dot_size or 3) / 2, 0.75)
    for i = 1, m do
        local t = (i - 1) / max(m - 1, 1)
        local r, g, b = draw.grad(geo.col1, geo.col2, t)
        draw.push_quad(v, xs[i] - rr, ys[i] - rr, xs[i] + rr, ys[i] + rr, r, g, b, geo.alpha)
    end
    return v
end

-- ============================================================
-- render (仕様書8.2節 ルートA: tempbuffer → obj.load)
-- ============================================================

-- verts(中心原点座標)をw×hのtempbufferへ一括描画し、オブジェクト画像に戻す。
-- opt: { blend = "add" 等 }
function draw.render_to_object(verts, w, h, opt)
    -- バッファは偶数サイズにする(中心原点のため、偶数だと整数座標が
    -- ピクセル境界に一致し、スナップ済みバーの幅が正確に揃う)
    w = math.floor(max(w, 2))
    h = math.floor(max(h, 2))
    if w % 2 == 1 then w = w + 1 end
    if h % 2 == 1 then h = h + 1 end
    obj.load("figure", "四角形", 0xffffff, 1)   -- 画像なしdrawpoly防御 (2.2節)
    obj.setoption("drawtarget", "tempbuffer", w, h)
    if opt and opt.blend then obj.setoption("blend", opt.blend) end
    if #verts > 0 then obj.drawpoly(verts) end
    if opt and opt.blend then obj.setoption("blend", "none") end
    obj.setoption("drawtarget", "framebuffer")
    obj.load("tempbuffer")
end

-- 統合レンダリング: SSAA (High/Export品質で2倍描画→縮小)
-- fx: { ssaa = 1|2 }
function draw.render_pipeline(verts, w, h, fx)
    local ss = (fx and fx.ssaa) or 1
    if ss ~= 1 and (w * ss > 4000 or h * ss > 4000) then ss = 1 end   -- SSAA上限
    local W = math.floor(max(w, 2) * ss)
    local H = math.floor(max(h, 2) * ss)
    -- 偶数サイズ化(中心原点で整数座標=ピクセル境界となり、バー幅が正確に揃う)
    if W % 2 == 1 then W = W + 1 end
    if H % 2 == 1 then H = H + 1 end

    local dv = verts
    if ss ~= 1 then
        dv = {}
        for i = 1, #verts do
            local p = verts[i]
            dv[i] = { p[1] * ss, p[2] * ss, p[3], p[4], p[5], p[6], p[7] }
        end
    end

    -- 本体をtempbufferへ
    obj.load("figure", "四角形", 0xffffff, 1)
    obj.setoption("drawtarget", "tempbuffer", W, H)
    if #dv > 0 then obj.drawpoly(dv) end
    obj.setoption("drawtarget", "framebuffer")
    obj.load("tempbuffer")


    -- SSAAダウンサンプル: 拡大画像をUVなしquadで1xへ縮小
    if ss ~= 1 then
        local w1 = math.floor(max(w, 2))
        local h1 = math.floor(max(h, 2))
        obj.setoption("drawtarget", "tempbuffer", w1, h1)
        obj.setoption("sampler", "clamp")
        obj.drawpoly(-w1 / 2, -h1 / 2, 0, w1 / 2, -h1 / 2, 0, w1 / 2, h1 / 2, 0, -w1 / 2, h1 / 2, 0)
        obj.setoption("drawtarget", "framebuffer")
        obj.load("tempbuffer")
    end
end

return draw
