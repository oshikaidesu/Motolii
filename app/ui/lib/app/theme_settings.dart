import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';

class ThemeSettings extends StatefulWidget {
  const ThemeSettings({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<ThemeSettings> createState() => _ThemeSettingsState();
}

class _ThemeSettingsState extends State<ThemeSettings> {
  String? _notice;
  bool _busy = false;
  int _request = 0;

  Future<void> _shareTheme() async {
    final c = widget.controller;
    await c.native('setPaneState', {
      'places': c.panePlaces.value,
      'drawer': c.deskDrawer.value,
      'theme': c.deskWork.value['theme'],
    });
  }

  Future<void> _load({bool reload = false}) async {
    if (_busy) return;
    final request = ++_request;
    var applied = false;
    setState(() {
      _busy = true;
      _notice = null;
    });
    try {
      String? path;
      if (reload) {
        path =
            EditorSession.map(widget.controller.deskWork.value['theme'])['path']
                as String?;
      } else {
        final picked = await widget.controller.native('pickImport', {
          'extensions': ['json'],
        });
        if (picked is List && picked.isNotEmpty) path = picked.first as String;
      }
      if (path == null || !mounted || request != _request) return;
      final file = File(path);
      if (await file.length() > 131072)
        throw const FormatException('Theme file must be smaller than 128 KB');
      final theme = EditorTheme.fromJson(jsonDecode(await file.readAsString()));
      if (!mounted || request != _request) return;
      final saved = widget.controller.storeDesk('theme', {
        'path': path,
        'data': theme.toJson(),
      });
      applied = true;
      await saved;
      await _shareTheme();
      if (mounted) setState(() => _notice = 'Loaded ${theme.name}');
    } catch (error) {
      if (mounted)
        setState(
          () => _notice = applied
              ? 'Theme loaded, but settings could not be saved: $error'
              : 'Theme unchanged: ${error is FormatException ? error.message : error}',
        );
    } finally {
      if (mounted && request == _request) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = EditorTheme.of(context);
    final source = EditorSession.map(widget.controller.deskWork.value['theme']);
    return EditorSection(
      'Color theme',
      Padding(
        padding: const EdgeInsets.all(EditorMetrics.s6),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              theme.name,
              key: const ValueKey('theme:name'),
              style: theme.text,
            ),
            const SizedBox(height: EditorMetrics.s4),
            Wrap(
              spacing: EditorMetrics.s4,
              runSpacing: EditorMetrics.s4,
              children: [
                EditorButton(
                  'Load JSON…',
                  _busy ? null : () => _load(),
                  key: const ValueKey('theme:load'),
                ),
                EditorButton(
                  'Reload',
                  !_busy && source['path'] is String
                      ? () => _load(reload: true)
                      : null,
                  key: const ValueKey('theme:reload'),
                ),
                EditorButton('Copy JSON', () async {
                  await Clipboard.setData(ClipboardData(text: theme.json));
                  if (mounted) setState(() => _notice = 'Theme JSON copied');
                }, key: const ValueKey('theme:copy')),
                EditorButton(
                  'Default',
                  _busy
                      ? null
                      : () async {
                          await widget.controller.storeDesk('theme', null);
                          await _shareTheme();
                          if (mounted) setState(() => _notice = null);
                        },
                  key: const ValueKey('theme:default'),
                ),
              ],
            ),
            if (_notice != null)
              Padding(
                padding: const EdgeInsets.only(top: EditorMetrics.s4),
                child: Text(
                  _notice!,
                  style: TextStyle(
                    fontSize: EditorMetrics.dense,
                    color: theme.muted,
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
