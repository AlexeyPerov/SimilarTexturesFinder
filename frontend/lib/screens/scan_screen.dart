import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/services.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../app_state.dart';
import '../services/backend_runner.dart';
import '../services/config_storage.dart';
import '../services/diagnostic_log.dart';
import '../services/json_parser.dart';

class ScanScreen extends StatefulWidget {
  const ScanScreen({super.key, required this.appState});

  final AppState appState;

  @override
  State<ScanScreen> createState() => _ScanScreenState();
}

class _ScanScreenState extends State<ScanScreen> {
  final _folderCtrl = TextEditingController();
  final _logCtrl = TextEditingController();

  bool _running = false;

  static const int _maxScanLogUiChars = 400 * 1024;
  static const int _maxToolStreamChars = 350 * 1024;
  static const int _maxScanLogBufChars = _maxToolStreamChars * 3;
  static const int _scanLogUiFlushThrottleMs = 200;

  String _capForUi(String s) {
    if (s.length <= _maxScanLogUiChars) return s;
    final keep = _maxScanLogUiChars ~/ 2;
    final removed = s.length - _maxScanLogUiChars;
    return '${s.substring(0, keep)}\n\n… [truncated $removed characters for UI] …\n\n${s.substring(s.length - keep)}';
  }

  String _capToolOutput(String s, int maxChars) {
    if (s.length <= maxChars) return s;
    final half = maxChars ~/ 2;
    final removed = s.length - maxChars;
    return '${s.substring(0, half)}\n\n[truncated $removed characters]\n\n${s.substring(s.length - half)}';
  }

  void _rollScanLogBufferIfNeeded(StringBuffer buf) {
    final s = buf.toString();
    if (s.length <= _maxScanLogBufChars) return;
    buf
      ..clear()
      ..write(_capToolOutput(s, _maxScanLogBufChars));
  }

  void _applyScanLogFromBuffer(StringBuffer buf) {
    if (!mounted) return;
    try {
      _logCtrl.text = _capForUi(buf.toString());
    } catch (e, st) {
      diagnosticLog('scan_log_ui_update_failed', error: e, stackTrace: st);
    }
  }

  @override
  void initState() {
    super.initState();
    final saved = widget.appState.lastScanInputFolder;
    if (saved != null &&
        saved.isNotEmpty &&
        Directory(saved).existsSync()) {
      _folderCtrl.text = saved;
    }
  }

  @override
  void dispose() {
    _folderCtrl.dispose();
    _logCtrl.dispose();
    super.dispose();
  }

  /// Directory URL seed for the folder sheet (persisted path, else current field).
  String? _initialDirectoryForPicker() {
    for (final candidate in [
      widget.appState.lastScanInputFolder,
      _folderCtrl.text.trim(),
    ]) {
      if (candidate != null && candidate.isNotEmpty && Directory(candidate).existsSync()) {
        return candidate;
      }
    }
    return null;
  }

  Future<void> _pickFolder() async {
    try {
      final path = await FilePicker.platform.getDirectoryPath(
        dialogTitle: 'Select folder to scan',
        initialDirectory: _initialDirectoryForPicker(),
        lockParentWindow: true,
      );
      if (!mounted) return;
      if (path != null) {
        setState(() => _folderCtrl.text = path);
        await widget.appState.setLastScanInputFolder(path);
      }
    } on PlatformException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Could not open folder picker: ${e.message ?? e.code}')),
      );
    }
  }

  Future<void> _runScan() async {
    final input = _folderCtrl.text.trim();
    if (input.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Choose an input folder')),
      );
      return;
    }
    if (!Directory(input).existsSync()) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Input folder does not exist')),
      );
      return;
    }

    await widget.appState.setLastScanInputFolder(input);

    final exe = resolveTextureToolExecutable(
      override: widget.appState.textureToolExecutableOverride,
    );
    if (exe == null) {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('texture_tool not found'),
          content: const Text(
            'In Settings, use Backend (texture_tool) to choose the executable, '
            'set TEXTURE_TOOL_PATH, build the binary under backend/target/release/, '
            'or install texture_tool on PATH. See README.',
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('OK')),
          ],
        ),
      );
      return;
    }

    setState(() {
      _running = true;
      _logCtrl.clear();
    });

    final logBuf = StringBuffer();
    await diagnosticLog('scan_start', data: {'input_len': input.length});

    try {
      await diagnosticLog('scan_phase', data: {'phase': 'write_config'});
      final cfgFile = await writeConfigToTempFile(widget.appState.config);
      final tmpDir = await getTemporaryDirectory();
      await Directory(tmpDir.path).create(recursive: true);
      final outFile = p.join(
        tmpDir.path,
        'texture_result_${DateTime.now().millisecondsSinceEpoch}.json',
      );

      await diagnosticLog('scan_phase', data: {
        'phase': 'config_ready',
        'cfg_base': p.basename(cfgFile.path),
      });

      logBuf
        ..writeln('Using: $exe')
        ..writeln('Config: ${cfgFile.path}')
        ..writeln('Output: $outFile')
        ..writeln();

      if (!mounted) return;
      _applyScanLogFromBuffer(logBuf);

      final threads = Platform.numberOfProcessors;
      await diagnosticLog(
        'scan_phase',
        data: {'phase': 'before_runTextureTool', 'threads': threads},
      );

      var lastLogUiFlush = DateTime.fromMillisecondsSinceEpoch(0);
      void flushScanLogUi({required bool force}) {
        final now = DateTime.now();
        if (!force &&
            now.difference(lastLogUiFlush).inMilliseconds < _scanLogUiFlushThrottleMs) {
          return;
        }
        lastLogUiFlush = now;
        _applyScanLogFromBuffer(logBuf);
      }

      final res = await runTextureTool(
        executable: exe,
        inputDir: input,
        outputJson: outFile,
        configPath: cfgFile.path,
        threads: threads > 0 ? threads : 4,
        onOutput: (chunk, {required isStderr}) {
          logBuf.write(chunk);
          _rollScanLogBufferIfNeeded(logBuf);
          flushScanLogUi(force: false);
        },
      );

      await diagnosticLog('scan_phase', data: {
        'phase': 'after_runTextureTool',
        'exit': res.exitCode,
        'stdout_len': res.stdoutText.length,
        'stderr_len': res.stderrText.length,
        'out_exists': File(outFile).existsSync(),
      });

      if (!mounted) return;
      flushScanLogUi(force: true);

      if (!mounted) return;

      if (res.success) {
        if (!mounted) return;
        await diagnosticLog('scan_phase', data: {'phase': 'before_parse'});
        final parsed = await ScanResult.parseFileAsync(outFile);
        await diagnosticLog('scan_phase', data: {
          'phase': 'parse_ok',
          'groups': parsed.groups.length,
        });
        widget.appState.setLastOutputs(
          configPath: cfgFile.path,
          resultPath: outFile,
          result: parsed,
        );
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Wrote $outFile')),
        );
      } else {
        if (!mounted) return;
        await diagnosticLog('scan_phase', data: {
          'phase': 'tool_nonzero_exit',
          'exit': res.exitCode,
        });
        if (!mounted) return;
        final dialogBody = _capToolOutput(
          res.stderrTail.isEmpty ? res.stdoutText : res.stderrTail,
          _maxToolStreamChars,
        );
        await showDialog<void>(
          context: context,
          builder: (ctx) => AlertDialog(
            title: Text('texture_tool exited ${res.exitCode}'),
            content: SingleChildScrollView(
              child: SelectableText(dialogBody),
            ),
            actions: [
              TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Close')),
            ],
          ),
        );
      }
    } catch (e, st) {
      await diagnosticLog('scan_failed', error: e, stackTrace: st);
      if (!mounted) return;
      try {
        logBuf.writeln();
        logBuf.writeln('Error: $e');
        logBuf.writeln('$st');
        _applyScanLogFromBuffer(logBuf);
      } catch (_) {}
      await showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Scan failed'),
          content: SingleChildScrollView(child: SelectableText('$e\n$st')),
          actions: [
            TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Close')),
          ],
        ),
      );
    } finally {
      if (mounted) setState(() => _running = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Scan', style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _folderCtrl,
                  decoration: const InputDecoration(
                    labelText: 'Input folder',
                    hintText: 'Folder to scan (recursive)',
                  ),
                ),
              ),
              const SizedBox(width: 8),
              FilledButton(onPressed: _running ? null : _pickFolder, child: const Text('Browse…')),
            ],
          ),
          const SizedBox(height: 12),
          FilledButton.icon(
            onPressed: _running ? null : _runScan,
            icon: const Icon(Icons.play_arrow),
            label: Text(_running ? 'Running…' : 'Run texture_tool'),
          ),
          if (_running) ...[
            const SizedBox(height: 8),
            Row(
              children: [
                const Expanded(child: LinearProgressIndicator()),
                const SizedBox(width: 12),
                OutlinedButton(
                  onPressed: () => killActiveTextureTool(),
                  child: const Text('Cancel'),
                ),
              ],
            ),
          ],
          const SizedBox(height: 16),
          Text('Log', style: Theme.of(context).textTheme.titleMedium),
          Expanded(
            child: DecoratedBox(
              decoration: BoxDecoration(
                border: Border.all(color: Theme.of(context).dividerColor),
                borderRadius: BorderRadius.circular(8),
              ),
              child: TextField(
                controller: _logCtrl,
                maxLines: null,
                expands: true,
                readOnly: true,
                textAlignVertical: TextAlignVertical.top,
                decoration: const InputDecoration(
                  border: InputBorder.none,
                  contentPadding: EdgeInsets.all(8),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
