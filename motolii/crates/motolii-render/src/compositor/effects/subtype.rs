//! 欄の性格を、名前ではなく shader の中での**使われ方**から読む。
//!
//! 借りた定規は次元解析(units-of-measure の推論): 座標に足せば平行移動、座標を割れば長さ、
//! `sin` に入れば角度、色に掛ければ量、`.a` に掛ければ不透明、hash の xor に入れば種、
//! ループの上限なら数。起点は Vism の ABI で固定された入力(fragment 位置・location 0 の uv・
//! `render_size` / `pass_index` の slot・texture の戻り・FieldIn / SurfaceIn の member)だけで、
//! 欄の名前は一切見ない。語彙は Blender の property subtype を借り、無い物(SEED / OPACITY /
//! COUNT / LEVEL)を足した。決まらなければ `None` — 嘘の性格は付けない。

use std::collections::BTreeMap;

use naga::{BinaryOperator as Op, Expression, Handle, MathFunction as Mf, Module, ScalarKind, Statement, TypeInner};

/// 性格。並びが優先順位(後ろほど強い証拠)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subtype { Count, Opacity, Factor, Level, Time, Translation, Distance, Angle, Seed }

impl Subtype {
    pub fn name(self) -> &'static str {
        match self {
            Self::Count => "COUNT", Self::Factor => "FACTOR", Self::Level => "LEVEL", Self::Time => "TIME",
            Self::Translation => "TRANSLATION", Self::Distance => "DISTANCE", Self::Opacity => "OPACITY",
            Self::Angle => "ANGLE", Self::Seed => "SEED",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_uppercase().as_str() {
            "COUNT" => Self::Count, "FACTOR" | "PERCENTAGE" => Self::Factor, "LEVEL" => Self::Level,
            "TIME" | "TIME_ABSOLUTE" => Self::Time, "TRANSLATION" => Self::Translation,
            "DISTANCE" | "PIXEL" => Self::Distance, "OPACITY" => Self::Opacity, "ANGLE" => Self::Angle,
            "SEED" => Self::Seed, _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParamSubtype {
    pub subtype: Option<Subtype>,
    /// px / ° / "" (無次元)。None は不明。
    pub unit: Option<&'static str>,
    /// 同じ点の仲間(先頭の欄の番号)。
    pub group: Option<usize>,
}

// 起点の種類。
const COORD_PX: u16 = 1;
const COORD_NORM: u16 = 2;
const COLOR: u16 = 4;
const ALPHA: u16 = 8;
const TIME: u16 = 16;
const SIZE_PX: u16 = 32;
const INT: u16 = 64;
const PARAMS: u16 = 128;
const ABI_IN: u16 = 256;
const INV_SIZE: u16 = 512;

#[derive(Clone, Copy, Default, PartialEq)]
struct Taint { params: u64, src: u16, scaled: bool, literal: Option<f64> }

fn pure(p: Taint) -> bool { p.src & (COLOR | ALPHA | COORD_PX | COORD_NORM | TIME | SIZE_PX | INV_SIZE) == 0 }

impl Taint {
    fn union(self, o: Taint) -> Taint {
        Taint { params: self.params | o.params, src: self.src | o.src, scaled: self.scaled || o.scaled, literal: None }
    }
    fn has(self, bit: u16) -> bool { self.src & bit != 0 }
}

struct Analysis<'m> {
    module: &'m Module,
    info: naga::valid::ModuleInfo,
    param_count: usize,
    /// group 1 の uniform → 欄の番号。
    globals: BTreeMap<Handle<naga::GlobalVariable>, Taint>,
    params_struct: Option<Handle<naga::Type>>,
    abi_in: Vec<Handle<naga::Type>>,
    seeds: Vec<Vec<Taint>>,
    returns: Vec<Taint>,
    locals: Vec<BTreeMap<Handle<naga::LocalVariable>, Taint>>,
    /// 呼び出しの戻り(文の側で決まる)を式の側へ渡す。
    call_results: Vec<BTreeMap<Handle<Expression>, Taint>>,
    votes: Vec<BTreeMap<Subtype, (u32, Option<&'static str>)>>,
    tentative_groups: Vec<u64>,
    groups: Vec<u64>,
    /// 票を数えるのは収束した最後の周回だけ(途中の周回は値が揃っておらず、嘘の証拠が出る)。
    collect: bool,
}

/// 読めなかった時の答え: 全部不明。
pub fn unknown(param_count: usize) -> Vec<ParamSubtype> { vec![ParamSubtype::default(); param_count] }

/// 読み込み済みの module の中で、group 1 の binding 0..n が欄。
pub fn analyze_module(module: &Module, param_count: usize) -> Vec<ParamSubtype> { analyze(module, param_count) }

/// hook(field / surface)の snippet を読める形に: ABI の struct と欄の struct を前置する。
pub fn hook_stub(body: &str, stage_params: &str, names: &[String]) -> String {
    let mut stub = String::from(
        "struct FieldIn { frame_position: vec3f, normal: vec3f, params: array<vec4f, 3>, };\n\
         struct FieldOut { offset: vec3f, normal: vec3f, };\n\
         struct SurfaceIn { albedo: vec3f, normal: vec3f, view_dir: vec3f, world_position: vec3f, thickness: f32, params: array<vec4f, 3>, };\n\
         fn shade_surface(albedo: vec3f, normal: vec3f, view_dir: vec3f, world_position: vec3f, thickness: f32, knobs: vec4f) -> vec3f { return albedo * knobs.x; }\n\
         fn simplex3(v: vec3f) -> f32 { return v.x; }\n\
         fn fbm3(p: vec3f, octaves: u32) -> f32 { return select(p.x, 0.0, octaves == 0u); }\n",
    );
    stub.push_str(&format!("struct {stage_params} {{\n"));
    for name in names { stub.push_str(&format!("    {name}: f32,\n")); }
    stub.push_str("};\n");
    stub.push_str(body);
    stub
}

fn analyze(module: &Module, param_count: usize) -> Vec<ParamSubtype> {
    let Ok(info) = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all()).validate(module)
    else { return vec![ParamSubtype::default(); param_count] };
    let mut globals = BTreeMap::new();
    for (handle, var) in module.global_variables.iter() {
        let Some(binding) = &var.binding else {
            if var.name.as_deref() == Some("isf_FragNormCoord") { globals.insert(handle, Taint { src: COORD_NORM, ..Default::default() }); }
            continue;
        };
        if binding.group != 1 { continue }
        let i = binding.binding as usize;
        let taint = if i < param_count { Taint { params: 1 << i, ..Default::default() } }
            else if i == param_count { Taint { src: SIZE_PX, ..Default::default() } }
            else { Taint { src: TIME, ..Default::default() } };
        globals.insert(handle, taint);
    }
    let by_name = |n: &str| module.types.iter().find(|(_, t)| t.name.as_deref() == Some(n)).map(|(h, _)| h);
    let params_struct = by_name("FieldParams").or_else(|| by_name("SurfaceParams"));
    let abi_in = ["FieldIn", "SurfaceIn", "VsOut"].iter().filter_map(|n| by_name(n)).collect();
    let n = module.functions.len();
    let mut a = Analysis {
        module, info, param_count, globals, params_struct, abi_in,
        seeds: module.functions.iter().map(|(_, f)| vec![Taint::default(); f.arguments.len()]).collect(),
        returns: vec![Taint::default(); n],
        locals: vec![BTreeMap::new(); n],
        call_results: vec![BTreeMap::new(); n],
        votes: vec![BTreeMap::new(); param_count],
        tentative_groups: Vec::new(),
        groups: Vec::new(),
        collect: false,
    };
    for round in 0..4 {
        a.collect = round == 3;
        for (handle, function) in module.functions.iter() {
            a.function(handle.index(), function, None);
        }
        for (i, entry) in module.entry_points.iter().enumerate() {
            a.function(n + i, &entry.function, Some(entry));
        }
    }
    (0..param_count).map(|i| {
        let best = a.votes[i].iter().max_by_key(|(s, _)| **s);
        let group = a.groups.iter().find(|g| *g & (1 << i) != 0).map(|g| g.trailing_zeros() as usize);
        ParamSubtype { subtype: best.map(|(s, _)| *s), unit: best.and_then(|(_, (_, u))| *u), group }
    }).collect()
}

impl<'m> Analysis<'m> {
    fn is_int(&self, fun: usize, expr: Handle<Expression>) -> bool {
        let Some(finfo) = self.function_info(fun) else { return false };
        matches!(finfo[expr].ty.inner_with(&self.module.types),
            TypeInner::Scalar(s) | TypeInner::Vector { scalar: s, .. } if matches!(s.kind, ScalarKind::Sint | ScalarKind::Uint))
    }
    fn type_of(&self, fun: usize, expr: Handle<Expression>) -> Option<Handle<naga::Type>> {
        let finfo = self.function_info(fun)?;
        match finfo[expr].ty { naga::proc::TypeResolution::Handle(h) => Some(h), _ => None }
    }
    fn function_info(&self, fun: usize) -> Option<&naga::valid::FunctionInfo> {
        let n = self.module.functions.len();
        if fun < n {
            self.module.functions.iter().nth(fun).map(|(h, _)| &self.info[h])
        } else {
            Some(self.info.get_entry_point(fun - n))
        }
    }

    fn function(&mut self, fun: usize, f: &naga::Function, entry: Option<&naga::EntryPoint>) {
        let n = self.module.functions.len();
        if fun >= n && self.seeds.len() <= fun { self.seeds.resize(fun + 1, Vec::new()); self.returns.resize(fun + 1, Taint::default()); self.locals.resize(fun + 1, BTreeMap::new()); self.call_results.resize(fun + 1, BTreeMap::new()); }
        if self.seeds[fun].len() != f.arguments.len() { self.seeds[fun] = vec![Taint::default(); f.arguments.len()]; }
        // 引数の起点: ABI の struct、params の struct、location 0 の vec2(uv)、builtin position。
        for (i, arg) in f.arguments.iter().enumerate() {
            let mut t = self.seeds[fun][i];
            if Some(arg.ty) == self.params_struct { t.src |= PARAMS; }
            if self.abi_in.contains(&arg.ty) { t.src |= ABI_IN; }
            match &arg.binding {
                Some(naga::Binding::Location { location: 0, .. }) if entry.is_some() => t.src |= COORD_NORM,
                Some(naga::Binding::BuiltIn(naga::BuiltIn::Position { .. })) => t.src |= COORD_PX,
                _ => {}
            }
            match arg.name.as_deref() {
                Some("frame_position" | "world_position") => t.src |= COORD_PX,
                Some("albedo") => t.src |= COLOR,
                _ => {}
            }
            self.seeds[fun][i] = t;
        }
        // 式は配列の順に決まるが、呼び出しの戻りと local の値は文の側で決まる。式→文→式で 2 周。
        let mut taints: Vec<Taint> = vec![Taint::default(); f.expressions.len()];
        let mut ret = self.returns[fun];
        let collect = self.collect;
        for round in 0..2 {
            self.collect = collect && round == 1;
            for (handle, expr) in f.expressions.iter() {
                let t = self.expression(fun, &taints, handle, expr);
                taints[handle.index()] = t;
            }
            self.block(fun, f, &f.body, &mut taints, &mut ret);
        }
        self.collect = collect;
        self.returns[fun] = ret;
    }

    fn block(&mut self, fun: usize, f: &naga::Function, block: &naga::Block, taints: &mut Vec<Taint>, ret: &mut Taint) {
        for stmt in block.iter() {
            match stmt {
                Statement::Store { pointer, value } => {
                    if let Some(local) = self.local_of(f, *pointer) {
                        let t = taints[value.index()];
                        let slot = self.locals[fun].entry(local).or_default();
                        *slot = slot.union(t);
                    }
                }
                Statement::Call { function, arguments, result } => {
                    let callee = function.index();
                    for (i, arg) in arguments.iter().enumerate() {
                        if let Some(seed) = self.seeds[callee].get_mut(i) { *seed = seed.union(taints[arg.index()]); }
                    }
                    if let Some(r) = result {
                        let mut t = self.returns[callee];
                        for arg in arguments { t = t.union(taints[arg.index()]); }
                        taints[r.index()] = t;
                        self.call_results[fun].insert(*r, t);
                    }
                }
                Statement::Return { value: Some(v) } => { *ret = ret.union(taints[v.index()]); }
                Statement::Block(b) => self.block(fun, f, b, taints, ret),
                Statement::If { accept, reject, .. } => { self.block(fun, f, accept, taints, ret); self.block(fun, f, reject, taints, ret); }
                Statement::Loop { body, continuing, .. } => { self.block(fun, f, body, taints, ret); self.block(fun, f, continuing, taints, ret); }
                Statement::Switch { cases, .. } => { for c in cases { self.block(fun, f, &c.body, taints, ret); } }
                _ => {}
            }
        }
    }

    fn local_of(&self, f: &naga::Function, mut pointer: Handle<Expression>) -> Option<Handle<naga::LocalVariable>> {
        loop {
            match &f.expressions[pointer] {
                Expression::LocalVariable(l) => return Some(*l),
                Expression::Access { base, .. } | Expression::AccessIndex { base, .. } => pointer = *base,
                _ => return None,
            }
        }
    }

    fn vote(&mut self, params: u64, s: Subtype, unit: Option<&'static str>) {
        if !self.collect { return }
        for i in 0..self.param_count {
            if params & (1 << i) != 0 {
                let e = self.votes[i].entry(s).or_insert((0, unit));
                e.0 += 1;
                if e.1.is_none() { e.1 = unit; }
            }
        }
    }

    fn expression(&mut self, fun: usize, taints: &[Taint], handle: Handle<Expression>, expr: &Expression) -> Taint {
        let t = |h: &Handle<Expression>| taints[h.index()];
        match expr {
            Expression::Literal(l) => Taint { literal: match l { naga::Literal::F32(v) => Some(*v as f64), naga::Literal::F64(v) => Some(*v), _ => None }, ..Default::default() },
            Expression::FunctionArgument(i) => self.seeds[fun].get(*i as usize).copied().unwrap_or_default(),
            Expression::GlobalVariable(g) => self.globals.get(g).copied().unwrap_or_default(),
            Expression::LocalVariable(l) => self.locals[fun].get(l).copied().unwrap_or_default(),
            Expression::Load { pointer } => t(pointer),
            Expression::Access { base, index } => {
                let mut b = t(base);
                if b.has(ABI_IN) { b.src &= !ABI_IN; }
                b.union(Taint { params: 0, src: t(index).src & (INT | 0), ..Default::default() }).union(Taint { params: t(index).params, ..Default::default() })
            }
            Expression::AccessIndex { base, index } => {
                let b = t(base);
                if b.has(PARAMS) && b.params == 0 {
                    return Taint { params: 1 << *index, ..Default::default() };
                }
                if b.has(ABI_IN) {
                    let member = self.type_of(fun, *base).and_then(|ty| match &self.module.types[ty].inner {
                        TypeInner::Struct { members, .. } => members.get(*index as usize).and_then(|m| m.name.clone()), _ => None });
                    return match member.as_deref() {
                        Some("frame_position" | "world_position" | "position") => Taint { src: COORD_PX, ..Default::default() },
                        Some("uv") => Taint { src: COORD_NORM, ..Default::default() },
                        Some("albedo") => Taint { src: COLOR, ..Default::default() },
                        Some("params") => Taint { src: PARAMS, ..Default::default() },
                        _ => Taint::default(),
                    };
                }
                if b.has(COLOR) && *index == 3 && !b.has(ALPHA) {
                    let mut a = b; a.src = (a.src & !COLOR) | ALPHA; return a;
                }
                b
            }
            Expression::Swizzle { vector, .. } | Expression::Splat { value: vector, .. } => t(vector),
            Expression::Unary { expr, .. } | Expression::Derivative { expr, .. } | Expression::ArrayLength(expr) => t(expr),
            Expression::As { expr, kind, .. } => {
                let e = t(expr);
                if matches!(kind, ScalarKind::Sint | ScalarKind::Uint) && e.params != 0 { self.vote(e.params, Subtype::Count, None); }
                e
            }
            Expression::Compose { components, .. } => {
                let mut out = Taint::default();
                let mut singles = 0u64;
                let mut all_single = !components.is_empty();
                for c in components {
                    let ct = t(c);
                    out = out.union(ct);
                    if ct.params.count_ones() == 1 && ct.src == 0 { singles |= ct.params; } else { all_single = false; }
                }
                if all_single && singles.count_ones() >= 2 { self.tentative_groups.push(singles); }
                if let Some(ps) = self.params_struct { if self.type_of(fun, handle) == Some(ps) { out.src |= PARAMS; out.params = 0; } }
                out
            }
            Expression::Select { accept, reject, .. } => t(accept).union(t(reject)),
            Expression::Relational { argument, .. } => t(argument),
            Expression::ImageSample { coordinate, .. } | Expression::ImageLoad { coordinate, .. } => {
                let c = t(coordinate);
                if c.params != 0 && !c.has(COORD_PX | COORD_NORM) { self.vote(c.params, Subtype::Translation, Some("")); }
                Taint { src: COLOR, ..Default::default() }
            }
            Expression::ImageQuery { .. } => Taint { src: SIZE_PX, ..Default::default() },
            Expression::Binary { op, left, right } => {
                let (li, ri) = (self.is_int(fun, *left), self.is_int(fun, *right));
                self.binary(*op, t(left), t(right), li, ri)
            }
            Expression::Math { fun: mf, arg, arg1, arg2, .. } => self.math(*mf, t(arg), arg1.as_ref().map(t), arg2.as_ref().map(t)),
            Expression::CallResult(_) => self.call_results[fun].get(&handle).copied().unwrap_or_default(),
            _ => Taint::default(),
        }
    }

    fn binary(&mut self, op: Op, l: Taint, r: Taint, l_int: bool, r_int: bool) -> Taint {
        let mut out = l.union(r);
        let sides = [(l, r, r_int, true), (r, l, l_int, false)];
        for (p, o, o_int, p_is_left) in sides {
            // 欄の値そのもの(色や座標がまだ混ざっていない)だけが証拠になる。
            if p.params == 0 || !pure(p) { continue }
            let obs: Option<(Subtype, Option<&'static str>)> = match op {
                Op::Add | Op::Subtract => {
                    if o.has(COORD_PX) { Some((if p.scaled { Subtype::Distance } else { Subtype::Translation }, Some("px"))) }
                    else if o.has(COORD_NORM) { Some((Subtype::Translation, Some(""))) }
                    else if o.has(TIME) { Some((Subtype::Time, None)) }
                    else if o.has(COLOR) { Some((Subtype::Level, None)) }
                    else if o_int && o.params == 0 { Some((Subtype::Count, None)) }
                    else { None }
                }
                Op::Multiply => {
                    if o.literal.is_some_and(|v| (v - std::f64::consts::PI / 180.0).abs() < 1e-6) { Some((Subtype::Angle, Some("°"))) }
                    else if o.has(ALPHA) { Some((Subtype::Opacity, Some("%"))) }
                    else if o.has(COLOR) { Some((Subtype::Factor, None)) }
                    else if o.has(INV_SIZE) { Some((Subtype::Distance, Some("px"))) }
                    else if o.has(COORD_PX | COORD_NORM | SIZE_PX) { Some((Subtype::Factor, Some(""))) }
                    else if o.has(TIME) { Some((Subtype::Time, None)) }
                    else { None }
                }
                Op::Divide => {
                    if !p_is_left && o.has(COORD_PX | SIZE_PX) { Some((Subtype::Distance, Some("px"))) }
                    else if !p_is_left && o.has(COORD_NORM) { Some((Subtype::Distance, Some(""))) }
                    else if p_is_left && o.has(COORD_PX | COORD_NORM | SIZE_PX) { Some((Subtype::Factor, Some(""))) }
                    else { None }
                }
                Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual | Op::Equal | Op::NotEqual => {
                    if o.has(COORD_PX) { Some((Subtype::Distance, Some("px"))) }
                    else if o.has(COORD_NORM) { Some((Subtype::Distance, Some(""))) }
                    else if o.has(COLOR) { Some((Subtype::Level, None)) }
                    else if o_int { Some((Subtype::Count, None)) }
                    else { None }
                }
                Op::ExclusiveOr | Op::ShiftLeft | Op::ShiftRight | Op::And | Op::InclusiveOr => Some((Subtype::Seed, None)),
                _ => None,
            };
            if let Some((s, unit)) = obs {
                self.vote(p.params, s, unit);
                if matches!(op, Op::Add | Op::Subtract) && o.has(COORD_PX | COORD_NORM) {
                    let confirmed: Vec<u64> = self.tentative_groups.iter().copied().filter(|g| p.params & g == *g).collect();
                    for g in confirmed { if !self.groups.contains(&g) { self.groups.push(g); } }
                }
            }
        }
        // 座標を欄で割った物は比率(px ではなくなる)。1 / render_size は px → 比率の換算係数。
        // 掛けた相手が数でも欄でもなければ「伸ばされた」。
        if op == Op::Divide && l.has(COORD_PX) && r.params != 0 { out.src = (out.src & !COORD_PX) | COORD_NORM; }
        if op == Op::Divide && !l.has(SIZE_PX) && r.has(SIZE_PX) && r.params == 0 { out.src = (out.src & !SIZE_PX) | INV_SIZE; }
        if op == Op::Multiply {
            let scaled_by_vector = |p: Taint, o: Taint| p.params != 0 && o.params == 0 && o.src == 0 && o.literal.is_none();
            if scaled_by_vector(l, r) || scaled_by_vector(r, l) { out.scaled = true; }
        }
        out.literal = None;
        out
    }

    fn math(&mut self, mf: Mf, a: Taint, b: Option<Taint>, c: Option<Taint>) -> Taint {
        let mut out = a;
        if let Some(b) = b { out = out.union(b); }
        if let Some(c) = c { out = out.union(c); }
        match mf {
            Mf::Sin | Mf::Cos | Mf::Tan | Mf::Radians | Mf::Degrees | Mf::Atan2 | Mf::Asin | Mf::Acos => {
                if a.params != 0 && pure(a) { self.vote(a.params, Subtype::Angle, Some("°")); }
            }
            Mf::Mix => {
                if let Some(c) = c { if c.params != 0 && pure(c) { self.vote(c.params, Subtype::Factor, Some("")); } }
            }
            Mf::Pow => { if let Some(b) = b { if b.params != 0 && pure(b) && a.has(COLOR) { self.vote(b.params, Subtype::Level, None); } } }
            Mf::Step | Mf::SmoothStep => {
                let x = c.or(b).unwrap_or_default();
                for edge in [Some(a), b].into_iter().flatten() {
                    if edge.params != 0 && pure(edge) {
                        if x.has(COLOR) { self.vote(edge.params, Subtype::Level, None); }
                        else if x.has(COORD_PX) { self.vote(edge.params, Subtype::Distance, Some("px")); }
                        else if x.has(COORD_NORM) { self.vote(edge.params, Subtype::Distance, Some("")); }
                    }
                }
            }
            _ => {}
        }
        out.literal = None;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pass(body: &str, n: usize) -> Vec<ParamSubtype> {
        let mut src = String::new();
        for i in 0..n { src.push_str(&format!("@group(1) @binding({i}) var<uniform> p{i}: f32;\n")); }
        src.push_str(&format!("@group(1) @binding({n}) var<uniform> render_size: vec2f;\n@group(1) @binding({}) var<uniform> pass_index: f32;\n", n + 1));
        src.push_str("@group(0) @binding(0) var tex: texture_2d<f32>;\n@group(0) @binding(1) var samp: sampler;\n");
        src.push_str("@fragment fn fs_main(@builtin(position) position: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {\n");
        src.push_str(body);
        src.push_str("\n}\n");
        analyze_module(&super::super::catalog::parse_wgsl(&src).unwrap(), n)
    }
    fn sub(v: &[ParamSubtype], i: usize) -> Option<Subtype> { v[i].subtype }

    #[test]
    fn added_to_a_coordinate_is_a_translation_in_px() {
        let v = pass("let q = position.xy + vec2f(p0, p1); return textureSample(tex, samp, q / render_size);", 2);
        assert_eq!((sub(&v, 0), v[0].unit, v[0].group), (Some(Subtype::Translation), Some("px"), Some(0)));
        assert_eq!(v[1].group, Some(0), "x と y は 1 つの点");
    }

    #[test]
    fn dividing_a_coordinate_is_a_distance() {
        let v = pass("let q = position.xy / max(p0, 1e-3); return vec4f(q, 0.0, 1.0);", 1);
        assert_eq!((sub(&v, 0), v[0].unit), (Some(Subtype::Distance), Some("px")));
    }

    #[test]
    fn trig_is_an_angle_and_color_gain_is_a_factor_and_alpha_is_opacity() {
        let v = pass("let c = textureSample(tex, samp, uv); let s = sin(p0); return vec4f(c.rgb * p1 * s, c.a * p2);", 3);
        assert_eq!(sub(&v, 0), Some(Subtype::Angle));
        assert_eq!(sub(&v, 1), Some(Subtype::Factor));
        assert_eq!(sub(&v, 2), Some(Subtype::Opacity));
    }

    #[test]
    fn a_threshold_against_luminance_is_a_level_and_a_loop_bound_is_a_count() {
        let v = pass(
            "let c = textureSample(tex, samp, uv); let l = dot(c.rgb, vec3f(0.3, 0.59, 0.11)); let k = max(l - p0, 0.0);\n\
             var sum = 0.0; let n = i32(p1); for (var i = 0; i < n; i = i + 1) { sum = sum + k; } return vec4f(sum);", 2);
        assert_eq!(sub(&v, 0), Some(Subtype::Level));
        assert_eq!(sub(&v, 1), Some(Subtype::Count));
    }

    #[test]
    fn hashing_makes_a_seed_and_the_unknown_stays_unknown() {
        let v = pass("let h = (u32(p0) ^ 2654435761u) >> 3u; let g = p1 + 1.0; return vec4f(f32(h) + g);", 2);
        assert_eq!(sub(&v, 0), Some(Subtype::Seed));
        assert_eq!(sub(&v, 1), None, "証拠が無ければ性格を付けない");
    }

    #[test]
    fn a_hook_reads_its_params_through_the_abi_struct() {
        let names = ["ox", "oy", "oz", "size", "amount"].map(String::from);
        let stub = hook_stub(
            "fn field(in: FieldIn, p: FieldParams) -> FieldOut {\n  let q = (in.frame_position + vec3f(p.ox, p.oy, p.oz)) / max(p.size, 1e-3);\n  return FieldOut(q * p.amount, in.normal);\n}",
            "FieldParams", &names);
        let v = analyze_module(&super::super::catalog::parse_wgsl(&stub).unwrap(), names.len());
        assert_eq!(sub(&v, 0), Some(Subtype::Translation));
        assert_eq!(v[2].group, Some(0));
        assert_eq!(sub(&v, 3), Some(Subtype::Distance));
        assert_eq!(sub(&v, 4), Some(Subtype::Factor));
    }
}
