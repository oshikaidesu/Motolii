import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import 'history_records.dart';
import '../session/read_model.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../foundation/metrics.dart';

export 'blend_panel.dart';

class HistoryPanel extends StatefulWidget {
  const HistoryPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<HistoryPanel> createState() => _HistoryPanelState();
}

class _HistoryPanelState extends State<HistoryPanel> {
  bool _moving = false;
  EditorSession get c => widget.controller;
  int _depth(String key) => (c.state[key] as num? ?? 0).toInt();

  Future<void> _go(int target) async {
    if (_moving) return;
    setState(() => _moving = true);
    try {
      final total = _depth('undo') + _depth('redo');
      while (mounted && _depth('undo') != target) {
        final before = _depth('undo');
        final op = target < before ? 'undo' : 'redo';
        if (!panelCan(c, op)) break;
        await c.command(op);
        if (_depth('undo') != before + (op == 'undo' ? -1 : 1) ||
            _depth('undo') + _depth('redo') != total)
          break;
      }
    } finally {
      if (mounted) setState(() => _moving = false);
    }
  }

  @override
  void initState() {
    super.initState();
    c.refreshHistory();
  }

  @override
  Widget build(BuildContext context) => DefaultTabController(
    length: 3,
    child: Column(
      children: [
        TabBar(
          labelPadding: const EdgeInsets.symmetric(
            horizontal: EditorMetrics.s6,
          ),
          labelStyle: const TextStyle(fontSize: EditorMetrics.font),
          indicatorColor: EditorTheme.accent,
          onTap: (_) => c.refreshHistory(),
          tabs: const [
            Tab(text: 'Edits'),
            Tab(text: 'Records'),
            Tab(text: 'Checkpoints'),
          ],
        ),
        Expanded(
          child: TabBarView(
            children: [
              _edits(context),
              HistoryRecords(controller: c, checkpoints: false),
              HistoryRecords(controller: c, checkpoints: true),
            ],
          ),
        ),
      ],
    ),
  );

  Widget _edits(BuildContext context) => AnimatedBuilder(
    animation: c.document,
    builder: (context, _) {
      final back = _depth('undo'), forward = _depth('redo');
      final total = back + forward;
      return Column(
        children: [
          EditorBar(
            height: EditorMetrics.bar,
            children: [
              EditorIconButton(
                tooltip: 'Undo',
                constraints: EditorTheme.iconConstraints,
                padding: EdgeInsets.zero,
                iconSize: EditorMetrics.s16,
                onPressed: !_moving && back > 0 && panelCan(c, 'undo')
                    ? () => _go(back - 1)
                    : null,
                icon: const Icon(Icons.undo),
              ),
              EditorIconButton(
                tooltip: 'Redo',
                constraints: EditorTheme.iconConstraints,
                padding: EdgeInsets.zero,
                iconSize: EditorMetrics.s16,
                onPressed: !_moving && forward > 0 && panelCan(c, 'redo')
                    ? () => _go(back + 1)
                    : null,
                icon: const Icon(Icons.redo),
              ),
              const Spacer(),
              Text(
                '$back / $total',
                style: const TextStyle(
                  fontSize: EditorMetrics.font,
                  color: EditorTheme.muted,
                ),
              ),
              const SizedBox(width: EditorMetrics.s8),
            ],
          ),
          Expanded(
            child: ListView.builder(
              itemExtent: EditorMetrics.bar,
              itemCount: total + 1,
              itemBuilder: (context, index) {
                final step = total - index;
                final current = step == back;
                final future = step > back;
                final label = step == 0 ? 'History start' : 'Edit $step';
                final color = current
                    ? EditorTheme.accent
                    : future
                    ? EditorTheme.disabledInk
                    : EditorTheme.ink;
                final enabled =
                    !_moving &&
                    !current &&
                    panelCan(c, future ? 'redo' : 'undo');
                return Semantics(
                  selected: current,
                  child: Tooltip(
                    message: current
                        ? '$label · Current state'
                        : '${future ? 'Redo' : 'Undo'} ${(step - back).abs()} steps to $label',
                    child: Material(
                      color: current ? EditorTheme.raised : Colors.transparent,
                      child: InkWell(
                        onTap: enabled ? () => _go(step) : null,
                        hoverColor: EditorTheme.hover,
                        child: Row(
                          children: [
                            SizedBox(
                              width: EditorMetrics.s32,
                              child: Stack(
                                alignment: Alignment.center,
                                children: [
                                  Column(
                                    children: [
                                      Expanded(
                                        child: Container(
                                          width: EditorMetrics.s2,
                                          color: index == 0
                                              ? Colors.transparent
                                              : future
                                              ? EditorTheme.border
                                              : EditorTheme.accent,
                                        ),
                                      ),
                                      Expanded(
                                        child: Container(
                                          width: EditorMetrics.s2,
                                          color: step == 0
                                              ? Colors.transparent
                                              : future
                                              ? EditorTheme.border
                                              : EditorTheme.accent,
                                        ),
                                      ),
                                    ],
                                  ),
                                  Container(
                                    width: EditorMetrics.s12,
                                    height: EditorMetrics.s12,
                                    decoration: BoxDecoration(
                                      shape: BoxShape.circle,
                                      color: current
                                          ? EditorTheme.accent
                                          : EditorTheme.panel,
                                      border: Border.all(
                                        color: current
                                            ? EditorTheme.accent
                                            : color,
                                        width: EditorMetrics.s2,
                                      ),
                                    ),
                                  ),
                                ],
                              ),
                            ),
                            Expanded(
                              child: Text(
                                label,
                                overflow: TextOverflow.ellipsis,
                                style: TextStyle(
                                  fontSize: EditorMetrics.font,
                                  color: color,
                                ),
                              ),
                            ),
                            if (current)
                              const Icon(
                                Icons.arrow_left,
                                size: EditorMetrics.s16,
                                color: EditorTheme.accent,
                              ),
                            if (future)
                              const Icon(
                                Icons.redo,
                                size: EditorMetrics.s12,
                                color: EditorTheme.disabledInk,
                              ),
                            const SizedBox(width: EditorMetrics.s8),
                          ],
                        ),
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      );
    },
  );
}
