// aviutl_pixels.h: part of aviutl_fx.h. obj.getpixeldata and obj.putpixeldata as lua.txt says them (the AviUtl2 script
// skill's references/api.md, gasdyueer, MIT: "object", "tempbuffer" or "cache:..."; "rgba" or "bgra"; 8 bits each, its
// pointer, w, h), the pixels as sashimono-edit keeps them (MIT, objapi.py lua_getpixeldata and _pixel_source, native.py
// PixelData: straight colour, rows from the top; a buffer not there nil, 0, 0; the frame not read), the values the
// buffer's own as its shaders read them. Read: the buffer made straight again on a picture of 8 bits (the placing
// shader's way 2), read back by Filament's Renderer::readPixels as thing_fx.h reads a thing's picture, waited for where
// it is asked (Engine::flushAndWait runs the callback: details/Engine.cpp, purge). Put: any w x h pixels the pointer
// holds, as AviUtl2 takes them (sigma's Path_S hands its own, numbers packed in them for its shaders), a picture again,
// made premultiplied (way 1) in the buffer's place.

static AviutlBuf aviutl_bytes(AviutlRun& r, uint32_t w, uint32_t h) {   // (a picture of 8 bits that size, for one call: let go once used, Filament dropping it after what drew with it)
    AviutlBuf b;
    b.w = w, b.h = h;
    b.t = Texture::Builder().width(w).height(h).levels(1).format(Texture::InternalFormat::RGBA8).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC | Texture::Usage::UPLOADABLE).build(*r.s->engine);
    b.rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, b.t).build(*r.s->engine);
    return b;
}
static void aviutl_let_go(AviutlRun& r, const AviutlBuf& b) { r.s->engine->destroy(b.rt), r.s->engine->destroy(b.t); }
static void aviutl_swap(std::vector<uint8_t>& p) { for (size_t i = 0; i + 3 < p.size(); i += 4) std::swap(p[i], p[i + 2]); }   // (rgba and bgra)

// A buffer read back, as above: into `p`, its size said; false, none there.
static bool aviutl_read(AviutlRun& r, const std::string& target, std::vector<uint8_t>& p, uint32_t& w, uint32_t& h) {
    if (!r.bufs.count(target) || !r.bufs[target].t) return false;
    const AviutlBuf& from = r.bufs[target];
    const AviutlBuf b = aviutl_bytes(r, from.w, from.h);
    aviutl_lay(r, from, b.rt, b.w, b.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1, 0, 2);
    p.assign((size_t)b.w * b.h * 4, 0);
    r.s->renderer->readPixels(b.rt, 0, 0, b.w, b.h, backend::PixelBufferDescriptor::make(p.data(), p.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, [](void const*, size_t) {}));
    r.s->engine->flushAndWait();
    aviutl_let_go(r, b);
    return w = b.w, h = b.h, true;
}

static int aviutl_getpixeldata(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string target = aviutl_name(lua_isstring(L, 1) ? lua_tostring(L, 1) : "object"), order = lua_isstring(L, 2) ? lua_tostring(L, 2) : "rgba";
    if (order != "rgba" && order != "bgra") return aviutl_no(L, "obj.getpixeldata as " + order);
    if (target != "object" && target != "tempbuffer" && target.rfind("cache:", 0) != 0) return aviutl_no(L, "obj.getpixeldata of " + target);
    std::vector<uint8_t>& p = r.pixels[target];
    uint32_t w = 0, h = 0;
    if (!aviutl_read(r, target, p, w, h)) return lua_pushnil(L), lua_pushinteger(L, 0), lua_pushinteger(L, 0), 3;
    if (order == "bgra") aviutl_swap(p);
    r.order[p.data()] = order;
    return lua_pushlightuserdata(L, p.data()), lua_pushinteger(L, w), lua_pushinteger(L, h), 3;
}

// obj.pixeloption as lua.txt has it: "get", the buffer obj.getpixel reads ("obj" or "frm"), read again once said again
// (TextBackplate.anm2's maker: "get を指定し直すことで getpixel 用キャッシュも更新する"); "type", how a pixel is told
// ("col" or "rgb"); "put" and "blend" kept, nothing put by pixel yet.
static int aviutl_pixeloption(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string n = aviutl_str(L, 1);
    if (n == "get") r.pixelFrom = aviutl_name(aviutl_str(L, 2)), r.pixelRead = false;
    else if (n == "type") r.pixelType = aviutl_str(L, 2);
    else if (n != "put" && n != "blend") return aviutl_no(L, "obj.pixeloption " + n);
    return 0;
}
// obj.getpixel: none asked, the size of what it reads; (x, y[, "col" or "rgb"]) one pixel as sashimono-edit tells it
// (MIT, objapi.py _pixel): straight, its colour and alpha 0 to 1, or red, green, blue 0 to 255 and alpha; past the
// picture 0, 0. Read back once, until obj.pixeloption "get" says it again.
static int aviutl_getpixel(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    if (!lua_gettop(L)) return lua_pushinteger(L, r.bufs[r.pixelFrom].w), lua_pushinteger(L, r.bufs[r.pixelFrom].h), 2;
    if (!r.pixelRead) r.pixelRead = true, r.pixelW = r.pixelH = 0, aviutl_read(r, r.pixelFrom, r.pixelSeen, r.pixelW, r.pixelH);
    const long x = (long)std::floor(aviutl_num(L, 1)), y = (long)std::floor(aviutl_num(L, 2));
    const std::string kind = lua_isstring(L, 3) ? lua_tostring(L, 3) : r.pixelType;
    if (x < 0 || y < 0 || x >= (long)r.pixelW || y >= (long)r.pixelH) return lua_pushinteger(L, 0), lua_pushnumber(L, 0), 2;
    const uint8_t* p = r.pixelSeen.data() + ((size_t)y * r.pixelW + (size_t)x) * 4;
    if (kind == "rgb") return lua_pushinteger(L, p[0]), lua_pushinteger(L, p[1]), lua_pushinteger(L, p[2]), lua_pushnumber(L, p[3] / 255.0), 4;
    return lua_pushinteger(L, (p[0] << 16) | (p[1] << 8) | p[2]), lua_pushnumber(L, p[3] / 255.0), 2;
}

static int aviutl_putpixeldata(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string target = aviutl_name(aviutl_str(L, 1));
    if (target != "object" && target != "tempbuffer" && target.rfind("cache:", 0) != 0) return aviutl_no(L, "obj.putpixeldata into " + target);
    if (lua_type(L, 2) != LUA_TLIGHTUSERDATA) return aviutl_no(L, "obj.putpixeldata of a " + std::string(luaL_typename(L, 2)));
    const uint8_t* data = (const uint8_t*)lua_touserdata(L, 2);
    const uint32_t w = (uint32_t)aviutl_num(L, 3), h = (uint32_t)aviutl_num(L, 4);
    if (!data || !w || !h || w > 8192 || h > 8192) return 0;
    const std::string order = lua_isstring(L, 5) ? lua_tostring(L, 5) : r.order.count(data) ? r.order[data] : "rgba";
    auto* p = new std::vector<uint8_t>(data, data + (size_t)w * h * 4);
    if (order == "bgra") aviutl_swap(*p);
    const AviutlBuf picture = aviutl_bytes(r, w, h);
    picture.t->setImage(*r.s->engine, 0, backend::PixelBufferDescriptor(p->data(), p->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, [](void*, size_t, void* u) { delete (std::vector<uint8_t>*)u; }, p));
    const AviutlBuf& to = aviutl_buf(r, target, w, h);
    aviutl_lay(r, picture, to.rt, to.w, to.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1, 0, 1);
    aviutl_let_go(r, picture);
    return 0;
}
