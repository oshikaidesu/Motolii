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

static int aviutl_getpixeldata(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string target = aviutl_name(lua_isstring(L, 1) ? lua_tostring(L, 1) : "object"), order = lua_isstring(L, 2) ? lua_tostring(L, 2) : "rgba";
    if (order != "rgba" && order != "bgra") return aviutl_no(L, "obj.getpixeldata as " + order);
    if (target != "object" && target != "tempbuffer" && target.rfind("cache:", 0) != 0) return aviutl_no(L, "obj.getpixeldata of " + target);
    if (!r.bufs.count(target) || !r.bufs[target].t) return lua_pushnil(L), lua_pushinteger(L, 0), lua_pushinteger(L, 0), 3;
    const AviutlBuf& from = r.bufs[target];
    const AviutlBuf b = aviutl_bytes(r, from.w, from.h);
    aviutl_lay(r, from, b.rt, b.w, b.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1, 0, 2);
    std::vector<uint8_t>& p = r.pixels[target];
    p.assign((size_t)b.w * b.h * 4, 0);
    r.s->renderer->readPixels(b.rt, 0, 0, b.w, b.h, backend::PixelBufferDescriptor::make(p.data(), p.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, [](void const*, size_t) {}));
    r.s->engine->flushAndWait();
    aviutl_let_go(r, b);
    if (order == "bgra") aviutl_swap(p);
    r.order[p.data()] = order;
    return lua_pushlightuserdata(L, p.data()), lua_pushinteger(L, b.w), lua_pushinteger(L, b.h), 3;
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
