//! 層の時刻 — 書いた時刻を、その層が実際に読む時刻へ。
//! 箱の順番の札がずらし、Loop が折り返し、移り方が前後のコマを覗く。
//! どれも時刻の純関数で、絵にも箱にも触らない。

use super::*;

impl StoreView<'_> {
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
    pub fn schedule_shift(&self, holder: LayerId, i: usize, n: usize, base: RationalTime) -> Result<RationalTime, StoreError> {
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
