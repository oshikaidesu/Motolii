// Part of panel_inspector_a_kit.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

class _CellState extends State<_Cell> {
  bool _edit = false, _hover = false, _bad = false, _focus = false;
  int _band = 1;
  bool _live = false;
  double _dy = 0;
  final _link = LayerLink();
  final _portal = OverlayPortalController();
  final _ctl = TextEditingController();
  final _node = FocusNode(debugLabel: 'cell');
  final _efn = FocusNode(debugLabel: 'cell-edit');
  Timer? _hold;

  @override
  void initState() {
    super.initState();
    _bad = widget.startBad;
    _efn.addListener(() {
      if (!_efn.hasFocus && _edit) {
        _commit(_ctl.text);
      }
    });
  }

  @override
  void dispose() {
    _hold?.cancel();
    _ctl.dispose();
    _node.dispose();
    _efn.dispose();
    super.dispose();
  }

  _P get p => widget.p;
  _M get m => widget.m;
  String get _label => p.label.isEmpty ? (m.rows[p.id]?.label ?? p.id) : '${m.rows[p.id]?.label ?? ''} ${p.label}'.trim();

  void _open() {
    if (!widget.enabled) {
      return;
    }
    _ctl.text = m.isMixed(p.id) ? '' : _fmtNum(m.at(p.id), p.dec);
    _ctl.selection = TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    setState(() {
      _edit = true;
      _bad = false;
    });
    _efn.requestFocus();
  }

  void _commit(String s) {
    final x = double.tryParse(s.trim().replaceAll(RegExp(r'[^0-9eE+\-.]'), ''));
    setState(() => _edit = false);
    if (x == null) {
      setState(() => _bad = s.trim().isNotEmpty);
      if (s.trim().isNotEmpty) {
        m.note('"${s.trim()}" is not a number: kept ${_fmtNum(m.at(p.id), p.dec)}');
      }
      return;
    }
    m.typed(p.id, x);
  }

  void _cancelEdit() {
    setState(() => _edit = false);
    _node.requestFocus();
  }

  void _start() {
    _live = true;
    _dy = 0;
    _band = 1;
    m.begin();
    _node.requestFocus();
    if (widget.g == _Gr.ladder) {
      _portal.show();
    }
  }

  void _update(DragUpdateDetails d) {
    if (!_live) {
      return;
    }
    if (widget.g == _Gr.ladder) {
      final k = HardwareKeyboard.instance;
      _dy += d.delta.dy;
      var b = (1 + (_dy / 20).round()).clamp(0, 3);
      if (k.isShiftPressed) {
        b = (b - 1).clamp(0, 3);
      }
      if (k.isAltPressed) {
        b = (b + 1).clamp(0, 3);
      }
      if (b != _band) {
        setState(() => _band = b);
      }
      m.add(p.id, d.delta.dx * _bands[b]);
    } else {
      m.add(p.id, d.delta.dx * _mult());
    }
  }

  void _stop() {
    if (!_live) {
      return;
    }
    _live = false;
    _portal.hide();
    final extra = widget.g == _Gr.ladder ? ' (${_bandWords[_band]})' : '';
    m.end('scrub $_label$extra');
    setState(() => _band = 1);
  }

  void _stepHold(int dir) {
    void once() {
      m.begin();
      m.add(p.id, dir * _mult());
      m.end('step $_label ${dir > 0 ? '+' : '-'}${_fmtNum(_mult(), _mult() < 1 ? 1 : 0)}');
    }

    once();
    var n = 0;
    _hold?.cancel();
    _hold = Timer(const Duration(milliseconds: 380), () {
      _hold = Timer.periodic(const Duration(milliseconds: 70), (_) {
        n++;
        m.begin();
        m.add(p.id, dir * _mult() * (n > 12 ? 5 : 1));
        m.end('hold $_label ${dir > 0 ? '+' : '-'}');
      });
    });
  }

  void _stepRelease() {
    _hold?.cancel();
    _hold = null;
  }

  @override
  Widget build(BuildContext context) {
    if (widget.ghost != null && p.kind != _K.num) {
      return Container(
        height: _cellH,
        alignment: Alignment.centerRight,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: Text('→ ${widget.ghost}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Role.of(N.g76, Role.changed))),
      );
    }
    if (p.kind == _K.toggle) {
      final on = m.v[p.id]! > .5;
      return FittedBox(
        fit: BoxFit.scaleDown,
        alignment: Alignment.centerRight,
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Opacity(
            opacity: widget.enabled ? 1 : .5,
            child: PillSwitch(
              on: on,
              onChanged: !widget.enabled
                  ? null
                  : (b) {
                      m.begin();
                      m.v[p.id] = b ? 1 : 0;
                      m.end('${m.rows[p.id]?.label}: ${b ? 'On' : 'Off'}');
                    },
            ),
          ),
          const SizedBox(width: 8),
          SizedBox(width: 18, child: Text(on ? 'On' : 'Off', style: T.label(!widget.enabled ? N.g63 : (on ? N.g100 : N.g76)))),
        ]),
      );
    }
    if (p.kind == _K.choice) {
      final i = m.v[p.id]!.round().clamp(0, p.options.length - 1);
      final off = !widget.enabled;
      bool? pin(_Pose x) => widget.pose == null ? null : widget.pose == x;
      return _Ctl(
        enabled: !off,
        hover: pin(_Pose.hover),
        focus: pin(_Pose.focus),
        pressed: pin(_Pose.press),
        onTap: () {
          m.begin();
          m.v[p.id] = ((i + 1) % p.options.length).toDouble();
          m.end('${m.rows[p.id]?.label}: ${p.options[(i + 1) % p.options.length]}');
        },
        builder: (_, h, f, pr) => Container(
          height: _cellH,
          clipBehavior: Clip.antiAlias,
          decoration: BoxDecoration(color: off ? N.g10 : (pr ? N.g15 : N.g07), borderRadius: BorderRadius.circular(4), border: Border.all(color: off ? const Color(0x00000000) : _faceLine(f ? _Pose.focus : (pr ? _Pose.press : (h ? _Pose.hover : _Pose.rest))))),
          child: CustomPaint(
            painter: off ? const _HatchPaint() : null,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(children: [
                Expanded(child: Text(p.options[i], maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(off ? N.g63 : N.g91))),
                const SizedBox(width: 4),
                Text('›', style: T.name(h || f || pr ? N.g95 : N.g63)),
              ]),
            ),
          ),
        ),
      );
    }
    final mixed = m.isMixed(p.id);
    final ladderDec = widget.g == _Gr.ladder && _live ? math.max(p.dec, _band >= 2 ? _band - 1 : 0) : p.dec;
    final dim = !widget.enabled;
    final forced = widget.pose;
    final pose = forced ?? (_bad ? _Pose.bad : (_edit ? _Pose.edit : (_live ? _Pose.press : (_focus ? _Pose.focus : (_hover && !dim ? _Pose.hover : _Pose.rest)))));
    final tagW = widget.tag == null || widget.tag!.isEmpty ? null : Text(widget.tag!, style: T.label(pose == _Pose.hover && widget.g == _Gr.label ? N.g100 : N.g76));
    final isChanged = m.changed(p.id) && (Role.grey || !mixed) && !dim && widget.ghost == null && pose != _Pose.edit;
    bool showTag0(double w) => tagW != null && widget.g != _Gr.stepper && w >= 66;
    final tagRoom = widget.tag == null ? 0.0 : _textW(widget.tag!, T.label(N.g76)) + 4;
    Widget value(double w) {
      // one narrowing rule, in this order: the value is always whole. (1) the unit goes first, (2) the value may shrink to 10 px (11 -> 10, FittedBox scaleDown), (3) only then decimals drop one by one; a value that is not zero never reads 0
      final tagOn = showTag0(w);
      final base = w - 2 - (tagOn ? 8 : 16) - (tagOn ? tagRoom : 0) - 1;
      final unitW = p.unit.isEmpty ? 0.0 : 4 + _textW(p.unit, T.label(N.g76));
      var text = mixed ? '—' : _fmtNum(m.at(p.id), ladderDec);
      var showUnit = p.unit.isNotEmpty && w >= 96 && !mixed;
      if (!mixed) {
        if (showUnit && _textW(text, T.value(N.g91)) * (10 / 11) + unitW > base) {
          showUnit = false;
        }
        for (var d = ladderDec; d > 0 && _textW(text, T.value(N.g91)) * (10 / 11) > base; d--) {
          text = _fmtNum(m.at(p.id), d - 1);
        }
      }
      if (forced == _Pose.edit) {
        return Align(alignment: Alignment.centerRight, child: ColoredBox(color: C.mode.withValues(alpha: .45), child: Text(text, maxLines: 1, style: T.value(N.g100))));
      }
      if (_edit) {
        return Focus(
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
              _cancelEdit();
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: EditableText(
            controller: _ctl,
            focusNode: _efn,
            autofocus: true,
            textAlign: TextAlign.right,
            style: T.value(N.g100),
            cursorColor: N.g95,
            backgroundCursorColor: N.g44,
            selectionColor: Role.selected.withValues(alpha: .45),
            keyboardType: TextInputType.number,
            onSubmitted: _commit,
            maxLines: 1,
          ),
        );
      }
      if (widget.ghost != null) {
        final x = double.tryParse(widget.ghost!), narrow = w < 80;
        return Align(alignment: Alignment.centerRight, child: Text(narrow ? (x == null ? widget.ghost! : _fmtNum(x, 0)) : '→ ${widget.ghost}', maxLines: 1, overflow: TextOverflow.clip, style: T.value(Role.of(N.g76, Role.changed))));
      }
      return Row(mainAxisAlignment: MainAxisAlignment.end, children: [
        if (pose == _Pose.bad) const ErrMark(size: 10, gap: 4),
        Flexible(child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerRight, child: Text(text, maxLines: 1, softWrap: false, textAlign: TextAlign.right, style: T.value(dim ? N.g63 : (mixed && !Role.grey ? Role.info : (m.changed(p.id) ? Role.of(N.g95, Role.changed) : N.g91)))))),
        if (showUnit) Padding(padding: const EdgeInsets.only(left: 4), child: Text(p.unit, style: T.label(N.g76))),
      ]);
    }

    Widget face(double w) {
      final showTag = tagW != null && widget.g != _Gr.stepper && w >= 66;
      final grabbable = !dim && widget.g != _Gr.label && widget.g != _Gr.stepper && !_edit;
      final hot = pose == _Pose.hover || pose == _Pose.press || pose == _Pose.focus;
      return MouseRegion(
          cursor: dim ? SystemMouseCursors.forbidden : (_edit ? SystemMouseCursors.text : SystemMouseCursors.resizeLeftRight),
          onEnter: (_) => setState(() => _hover = true),
          onExit: (_) => setState(() => _hover = false),
          child: CompositedTransformTarget(
            link: _link,
            child: OverlayPortal(
              controller: _portal,
              overlayChildBuilder: (_) => Stack(children: [
                CompositedTransformFollower(
                  link: _link,
                  showWhenUnlinked: false,
                  targetAnchor: Alignment.centerLeft,
                  followerAnchor: Alignment.centerRight,
                  offset: const Offset(-8, 0),
                  child: Align(alignment: Alignment.centerRight, child: _Ladder(band: _band)),
                ),
              ]),
              child: Container(
                height: _cellH,
                clipBehavior: Clip.antiAlias,
                padding: EdgeInsets.symmetric(horizontal: showTag ? 4 : 8),
                decoration: BoxDecoration(
                  color: pose == _Pose.press ? N.g15 : (dim ? N.g10 : N.g07),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: dim ? const Color(0x00000000) : _faceLine(pose)),
                ),
                child: CustomPaint(
                  painter: dim ? const _HatchPaint() : null,
                  child: Stack(fit: StackFit.expand, clipBehavior: Clip.none, children: [
                    Row(children: [
                      if (widget.g == _Gr.stepper) _StepBtn('−', visible: (pose == _Pose.hover || pose == _Pose.press) && !dim, onDown: () => _stepHold(-1), onUp: _stepRelease, rest: dim ? null : _grip(N.g38)),
                      if (showTag)
                        if (widget.g == _Gr.label)
                          _LabelHandle(m: m, p: p, child: Padding(padding: const EdgeInsets.only(right: 4), child: tagW))
                        else
                          Padding(padding: const EdgeInsets.only(right: 4), child: tagW),
                      Expanded(child: value(w)),
                      if (widget.g == _Gr.stepper) _StepBtn('+', visible: (pose == _Pose.hover || pose == _Pose.press) && !dim, onDown: () => _stepHold(1), onUp: _stepRelease),
                    ]),
                    if (grabbable) Positioned(left: showTag ? -4 : -5, top: 0, bottom: 0, child: Center(child: _grip(hot ? N.g63 : N.g38))),
                    if (isChanged) Positioned(left: showTag ? -5 : -7, top: 0, child: const CustomPaint(size: Size(4, 4), painter: _CornerPaint())),
                  ]),
                ),
              ),
            ),
          ),
        );
    }

    Widget wrapGestures(Widget child) {
      if (dim) {
        return child;
      }
      final labelMode = widget.g == _Gr.label;
      return Focus(
        focusNode: _node,
        onFocusChange: (f) => setState(() => _focus = f),
        onKeyEvent: (_, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _live) {
            _live = false;
            _portal.hide();
            m.cancel();
            return KeyEventResult.handled;
          }
          if (e is KeyDownEvent && (e.logicalKey == LogicalKeyboardKey.arrowUp || e.logicalKey == LogicalKeyboardKey.arrowDown)) {
            m.begin();
            m.add(p.id, (e.logicalKey == LogicalKeyboardKey.arrowUp ? 1 : -1) * _mult());
            m.end('arrow $_label');
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Listener(
          onPointerSignal: (e) {
            if (e is PointerScrollEvent && _hover) {
              m.begin();
              m.add(p.id, (e.scrollDelta.dy > 0 ? -1 : 1) * _mult());
              m.end('wheel $_label');
            }
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            // the label grammar has no double-click on the value, so a click opens the field at once
            onTap: _open,
            onDoubleTap: labelMode ? null : () => m.reset(p.id),
            onPanStart: widget.g == _Gr.label ? null : (_) => _start(),
            onPanUpdate: widget.g == _Gr.label ? null : _update,
            onPanEnd: widget.g == _Gr.label ? null : (_) => _stop(),
            onPanCancel: widget.g == _Gr.label ? null : _stop,
            child: child,
          ),
        ),
      );
    }

    return LayoutBuilder(builder: (context, box) => wrapGestures(face(box.maxWidth)));
  }
}

class _StepBtn extends StatefulWidget {
  const _StepBtn(this.glyph, {required this.visible, required this.onDown, required this.onUp, this.hot = false, this.down = false, this.rest});
  final String glyph;
  final Widget? rest; // what the slot shows while the button is hidden (a grip, so the field reads as draggable)
  final bool visible, hot, down; // hot / down: a specimen's hover / pressed
  final VoidCallback onDown, onUp;
  @override
  State<_StepBtn> createState() => _StepBtnState();
}

class _StepBtnState extends State<_StepBtn> {
  bool _d = false;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: 20,
        child: widget.visible
            ? GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () {},
                child: Listener(
                  onPointerDown: (_) {
                    setState(() => _d = true);
                    widget.onDown();
                  },
                  onPointerUp: (_) {
                    setState(() => _d = false);
                    widget.onUp();
                  },
                  onPointerCancel: (_) {
                    setState(() => _d = false);
                    widget.onUp();
                  },
                  // rest: bare glyph g63. hover: g20 fill, g95. pressed: g26 fill, g100.
                  child: Hov(builder: (_, h) {
                    final dn = _d || widget.down, hv = h || widget.hot;
                    return Container(color: dn ? N.g26 : (hv ? N.g20 : null), alignment: Alignment.center, child: Text(widget.glyph, style: T.name(dn ? N.g100 : (hv ? N.g95 : N.g63)).copyWith(fontSize: 13)));
                  }),
                ),
              )
            : (widget.rest == null ? null : Center(child: widget.rest)),
      );
}

/// A label that is a handle: drag scrubs, a double-click goes to the default.
class _LabelHandle extends StatefulWidget {
  const _LabelHandle({required this.m, required this.p, required this.child});
  final _M m;
  final _P p;
  final Widget child;
  @override
  State<_LabelHandle> createState() => _LabelHandleState();
}

class _LabelHandleState extends State<_LabelHandle> {
  bool _live = false;
  @override
  Widget build(BuildContext context) {
    final m = widget.m, p = widget.p;
    final lab = m.rows[p.id]?.label ?? p.id;
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onDoubleTap: () => m.reset(p.id),
        onPanStart: (_) {
          _live = true;
          m.begin();
        },
        onPanUpdate: (d) {
          if (_live) {
            m.add(p.id, d.delta.dx * _mult());
          }
        },
        onPanEnd: (_) {
          _live = false;
          m.end('scrub $lab (label)');
        },
        onPanCancel: () {
          _live = false;
          m.end('scrub $lab (label)');
        },
        child: widget.child,
      ),
    );
  }
}
