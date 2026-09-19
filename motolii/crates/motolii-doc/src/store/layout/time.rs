//! 層の時刻 — 書いた時刻を、その層が実際に読む時刻へ。
//! 箱の順番の札がずらし、Loop が折り返し、移り方が前後のコマを覗く。
//! どれも時刻の純関数で、絵にも箱にも触らない。

use super::*;

impl StoreView<'_> {
    /// 移り方(Transition)が遡るコマ数の最大。host は解析の入力(Blob の塊)をこのコマ数だけ前まで置く。
    /// 移り方は重なる(折り返しの移り方が前の時刻で組み、その時刻の避ける物がさらに前の形を混ぜる)ので、長い順に 2 つの和と遅れ 2 つ分。
    pub fn transition_reach(&self, t: RationalTime) -> Result<i64, StoreError> {
        let Some(comp) = self.composition()? else { return Ok(0) };
        let mut longest = [0.0f64; 2];
        let mut waits = 0.0f64;
        for layer in self.layers() {
            waits = waits.max(self.number(layer, TRANSITION_DELAY, 0.0, t)? + self.number(layer, STAGGER, 0.0, t)?);
            let d = self.number(layer, TRANSITION_DURATION, 0.0, t)?;
            if d > longest[0] {
                longest = [d, longest[0]];
            } else if d > longest[1] {
                longest[1] = d;
            }
        }
        Ok(((longest[0] + longest[1] + 2.0 * waits) * comp.fps.as_f64()).round() as i64 + 1)
    }

    /// Transition の標本: (時刻, 重み)。位置(t) = Σ (E(uₖ₊₁) − E(uₖ)) · 行き先(t − 遅れ − D·uₖ)。時刻はコマに丸める。
    /// 遅れがコマの途中なら、前後のコマへ重みを分ける(遅れが時刻で変わっても位置が跳ばない)。
    /// Duration も遅れも 0 なら空(今の行き先そのまま)。
    pub(crate) fn transition_samples(&self, layer: LayerId, t: RationalTime) -> Result<Vec<(RationalTime, f32)>, StoreError> {
        let Some(comp) = self.composition()? else { return Ok(Vec::new()) };
        let fps = comp.fps;
        let base = self.base_samples(layer, t)?;
        // 自分の移り方を持たない並ぶ子は、容器の移り方を借りる: 容器の箱が移る途中は、その見えている箱の中で並ぶ
        // (CSS で幅が移る間、中身は毎コマその幅で並び直すのと同じ。揃え・伸びは箱の大きさに線形なので、同じ重みで混ぜれば一致する)。
        if base.is_empty() && self.number(layer, TRANSITION_DELAY, 0.0, t)? <= 0.0 {
            if let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| self.display(*p, t).is_ok_and(|d| d != 0)) {
                return self.transition_samples(parent, t);
            }
        }
        let delay = self.transition_delay(layer, t, &base)? * fps.as_f64();
        if delay <= 1e-6 {
            return base.into_iter().map(|(back, w)| Ok((self.frame_time(back, t)?, w))).collect();
        }
        let base = if base.is_empty() { vec![(0.0, 1.0)] } else { base };
        let mut out = Vec::with_capacity(base.len() * 2);
        for (back, w) in base {
            let at = back + delay;
            let lo = at.floor();
            let frac = (at - lo) as f32;
            out.push((self.frame_time(lo, t)?, w * (1.0 - frac)));
            if frac > 1e-4 {
                out.push((self.frame_time(lo + 1.0, t)?, w * frac));
            }
        }
        Ok(out)
    }

    /// 遅れの無い標本: (遡るコマ数, 重み)。
    fn base_samples(&self, layer: LayerId, t: RationalTime) -> Result<Vec<(f64, f32)>, StoreError> {
        let duration = self.number(layer, TRANSITION_DURATION, 0.0, t)?;
        let Some(comp) = self.composition()? else { return Ok(Vec::new()) };
        let frames = (duration * comp.fps.as_f64()).round();
        if frames < 1.0 {
            return Ok(Vec::new());
        }
        let easing = self.choice(layer, TRANSITION_EASING, t)?;
        // コマごとに 1 つ(時刻をずらしても重みの形が変わらない、畳み込みとして滑らか)。長い移り方だけ間引く。
        let n = frames.min(120.0) as usize;
        Ok((0..n).map(|k| {
            let (u0, u1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
            ((frames * u0).round(), (ease(easing, u1) - ease(easing, u0)) as f32)
        }).collect())
    }

    /// 今のコマから `back` コマ前の時刻(0 より前は 0)。
    fn frame_time(&self, back: f64, t: RationalTime) -> Result<RationalTime, StoreError> {
        let Some(comp) = self.composition()? else { return Ok(t) };
        let now = t.try_to_frame_round(comp.fps).map_err(|e| StoreError::Property(e.to_string()))?;
        RationalTime::try_from_frame((now - back as i64).max(0), comp.fps).map_err(|e| StoreError::Property(e.to_string()))
    }

    /// 移り方の遅れ(秒): 自分の Transition Delay + 並べる親の Stagger が配る分。
    /// 配る分は、遅れと長さの窓より前(t − Stagger − Duration)に並んでいた場所の、起点からの距離で決める
    /// (GSAP の stagger が今居る場所で測るのと同じ)。動いている途中の位置で測ると、動くほど遅れが変わって戻る。
    fn transition_delay(&self, layer: LayerId, t: RationalTime, base: &[(f64, f32)]) -> Result<f64, StoreError> {
        let own = self.number(layer, TRANSITION_DELAY, 0.0, t)?.max(0.0);
        let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent else { return Ok(own) };
        let stagger = self.number(parent, STAGGER, 0.0, t)?.max(0.0);
        if stagger <= 0.0 || self.display(parent, t)? == 0 {
            return Ok(own);
        }
        let Some(comp) = self.composition()? else { return Ok(own) };
        let window = base.iter().map(|(back, _)| *back).fold(0.0, f64::max) + 1.0 + ((stagger + own) * comp.fps.as_f64()).ceil();
        let before = crate::doc::store::layout::frame::layout_frame(self, self.frame_time(window, t)?)?;
        let (Some(size), Some(slot)) = (before.sizes.get(&parent).copied(), before.slots.get(&layer).copied()) else { return Ok(own) };
        let p = glam::Vec2::from(slot.position);
        let (w, h) = (size[0].max(1e-3), size[1].max(1e-3));
        let corner = glam::vec2(CANVAS_MARGIN, CANVAS_MARGIN);
        let diagonal = glam::vec2(w, h).length();
        let from_centre = (p - corner - glam::vec2(w, h) * 0.5).length() / (diagonal * 0.5);
        let reach = match self.choice(parent, STAGGER_FROM, t)? {
            1 => from_centre,
            2 => (p - corner - glam::vec2(w, h)).length() / diagonal,
            3 => 1.0 - from_centre,
            _ => (p - corner).length() / diagonal,
        };
        Ok(own + stagger * f64::from(reach.clamp(0.0, 1.0)))
    }

    /// その層の時刻(順番の札でずれた後)。親の箱に Stagger があれば、層の順の位置(Stagger From: Start / Center /
    /// End / Edges、移り方の遅れと同じ語)に応じて最大 Stagger 秒ずれる。From End なら進む(早い子が先に着く)、
    /// でなければ遅れる(後の子が後から始まる)。入れ子は親のずれの上に積む。
    pub fn layer_time(&self, layer: LayerId, t: RationalTime) -> Result<RationalTime, StoreError> {
        let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent else { return Ok(t) };
        let base = self.layer_time(parent, t)?;
        let siblings = self.schedule_children(parent)?;
        let Some(i) = siblings.iter().position(|&s| s == layer) else { return Ok(base) };
        self.schedule_shift(parent, i, siblings.len(), base)
    }

    /// 繰り返しで畳んだ時刻(CSS の animation-direction: normal は t mod D、reverse は D − (t mod D)、alternate は
    /// 奇数回目を逆向きに、alternate-reverse はその逆)。Loop Duration が 0 ならそのまま。
    pub fn looped_time(&self, layer: LayerId, t: RationalTime) -> Result<RationalTime, StoreError> {
        let duration = self.number(layer, LOOP_DURATION, 0.0, t)?;
        if duration <= 1e-9 {
            return Ok(t);
        }
        let seconds = t.as_seconds_f64();
        let round = (seconds / duration).floor();
        let along = seconds - round * duration;
        let odd = round.rem_euclid(2.0) >= 1.0;
        let local = match self.choice(layer, LOOP_DIRECTION, t)? {
            1 => duration - along,
            2 => if odd { duration - along } else { along },
            3 => if odd { along } else { duration - along },
            _ => along,
        };
        RationalTime::try_new((local * 1_000_000.0).round() as i64, 1_000_000).map_err(|e| StoreError::Property(format!("loop: {e}")))
    }

    /// 箱 `holder` の順番の札で、n 個のうち i 番目の時刻。箱の子の層にも、文字の Split の単位にも同じ法。
    pub(crate) fn schedule_shift(&self, holder: LayerId, i: usize, n: usize, base: RationalTime) -> Result<RationalTime, StoreError> {
        let stagger = match self.value_at(holder, &PropertyId::new(STAGGER)?, base)? {
            Some(Value::F64(v)) if v > 1e-9 => v,
            _ => return Ok(base),
        };
        let along = if n > 1 { i as f64 / (n - 1) as f64 } else { 0.0 };
        let from_centre = (along - 0.5).abs() * 2.0;
        let reach = match self.choice(holder, STAGGER_FROM, base)? {
            1 => from_centre,
            2 => 1.0 - along,
            3 => 1.0 - from_centre,
            _ => along,
        };
        let off = RationalTime::try_new((reach * stagger * 1_000_000.0).round() as i64, 1_000_000)
            .map_err(|e| StoreError::Property(format!("stagger: {e}")))?;
        let shifted = if self.choice(holder, FROM_END, base)? == 1 { base.try_add(off) } else { base.try_sub(off) }
            .map_err(|e| StoreError::Property(format!("stagger: {e}")))?;
        Ok(if shifted.as_seconds_f64() < 0.0 { RationalTime::ZERO } else { shifted })
    }

    /// 箱の子を層の順に(順番の札が読む)。書類の版ごとに覚える。
    fn schedule_children(&self, parent: LayerId) -> Result<std::sync::Arc<Vec<LayerId>>, StoreError> {
        let key = (self.revision_key(), parent);
        if let Some(hit) = self.layout_memo().borrow().kids.get(&key).cloned() {
            return Ok(hit);
        }
        let mut kids: Vec<(i16, LayerId)> = Vec::new();
        for layer in self.layers() {
            if self.attrs(layer)?.unwrap_or_default().parent == Some(parent) {
                if let Some(meta) = self.meta(layer)? {
                    kids.push((meta.order, layer));
                }
            }
        }
        kids.sort();
        let out = std::sync::Arc::new(kids.into_iter().map(|(_, l)| l).collect::<Vec<_>>());
        let mut scratch = self.layout_memo().borrow_mut();
        if scratch.kids.len() > 512 {
            scratch.kids.clear();
        }
        scratch.kids.insert(key, out.clone());
        Ok(out)
    }
}
