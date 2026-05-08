import 'package:file_picker/file_picker.dart';
import 'package:flutter/services.dart';
import 'package:flutter/material.dart';

import '../app_state.dart';
import '../models/config.dart';
import '../services/backend_runner.dart';
import '../services/config_storage.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key, required this.appState});

  final AppState appState;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  late AnalysisConfig _cfg;
  late TextEditingController _thr;
  late TextEditingController _wPhash;
  late TextEditingController _wSsim;
  late TextEditingController _wHist;
  late TextEditingController _hashAlg;
  late TextEditingController _phashDist;
  late TextEditingController _ssimThr;
  late TextEditingController _resize;
  late TextEditingController _histBins;
  late TextEditingController _histMethod;
  late TextEditingController _alphaThr;
  late TextEditingController _maxDecode;
  bool _controllersBound = false;

  @override
  void initState() {
    super.initState();
    _syncFromApp();
  }

  void _disposeControllers() {
    if (!_controllersBound) return;
    _thr.dispose();
    _wPhash.dispose();
    _wSsim.dispose();
    _wHist.dispose();
    _hashAlg.dispose();
    _phashDist.dispose();
    _ssimThr.dispose();
    _resize.dispose();
    _histBins.dispose();
    _histMethod.dispose();
    _alphaThr.dispose();
    _maxDecode.dispose();
  }

  void _syncFromApp() {
    _disposeControllers();
    _cfg = widget.appState.config.copy();
    _thr = TextEditingController(text: _cfg.threshold.toStringAsFixed(3));
    _wPhash = TextEditingController(text: _cfg.weights.phash.toString());
    _wSsim = TextEditingController(text: _cfg.weights.ssim.toString());
    _wHist = TextEditingController(text: _cfg.weights.histogram.toString());
    _hashAlg = TextEditingController(text: _cfg.hashAlgorithm);
    _phashDist = TextEditingController(text: '${_cfg.phashMaxDistance}');
    _ssimThr = TextEditingController(text: _cfg.ssimThreshold.toStringAsFixed(2));
    _resize = TextEditingController(text: '${_cfg.resizeSize}');
    _histBins = TextEditingController(text: '${_cfg.histBins}');
    _histMethod = TextEditingController(text: _cfg.histMethod);
    _alphaThr = TextEditingController(text: _cfg.alphaThreshold.toStringAsFixed(3));
    _maxDecode = TextEditingController(
      text: _cfg.maxDecodeDimensionPx?.toString() ?? '',
    );
    _controllersBound = true;
  }

  @override
  void dispose() {
    _disposeControllers();
    super.dispose();
  }

  void _apply() {
    setState(() {
      _cfg.threshold = double.tryParse(_thr.text.trim()) ?? _cfg.threshold;
      _cfg.weights = AnalysisWeights(
        phash: double.tryParse(_wPhash.text.trim()) ?? _cfg.weights.phash,
        ssim: double.tryParse(_wSsim.text.trim()) ?? _cfg.weights.ssim,
        histogram: double.tryParse(_wHist.text.trim()) ?? _cfg.weights.histogram,
      );
      _cfg.hashAlgorithm = _hashAlg.text.trim().isEmpty
          ? 'sha256'
          : _hashAlg.text.trim();
      _cfg.phashMaxDistance =
          int.tryParse(_phashDist.text.trim()) ?? _cfg.phashMaxDistance;
      _cfg.ssimThreshold =
          double.tryParse(_ssimThr.text.trim()) ?? _cfg.ssimThreshold;
      _cfg.resizeSize = int.tryParse(_resize.text.trim()) ?? _cfg.resizeSize;
      _cfg.histBins = int.tryParse(_histBins.text.trim()) ?? _cfg.histBins;
      _cfg.histMethod = _histMethod.text.trim().isEmpty
          ? 'correlation'
          : _histMethod.text.trim();
      _cfg.alphaThreshold =
          double.tryParse(_alphaThr.text.trim()) ?? _cfg.alphaThreshold;
      final m = _maxDecode.text.trim();
      _cfg.maxDecodeDimensionPx = m.isEmpty ? null : int.tryParse(m);
    });
    widget.appState.setConfig(_cfg.copy());
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Settings saved for next scan')),
      );
    }
  }

  void _resetDefaults() {
    setState(() {
      widget.appState.setConfig(AnalysisConfig.defaults());
      _syncFromApp();
    });
  }

  Future<void> _pickExecutable() async {
    try {
      final result = await FilePicker.platform.pickFiles(
        dialogTitle: 'Select texture_tool executable',
        type: FileType.any,
        allowMultiple: false,
        lockParentWindow: true,
      );
      if (!mounted) return;
      if (result == null || result.files.isEmpty) return;
      final path = result.files.single.path;
      if (path == null || path.isEmpty) return;
      await widget.appState.setTextureToolExecutableOverride(path);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Backend executable set to $path')),
      );
    } on PlatformException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Could not open file picker: ${e.message ?? e.code}'),
        ),
      );
    }
  }

  Future<void> _clearExecutableOverride() async {
    await widget.appState.setTextureToolExecutableOverride(null);
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Cleared saved executable path')),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.appState,
      builder: (context, _) {
        final resolvedExe = resolveTextureToolExecutable(
          override: widget.appState.textureToolExecutableOverride,
        );
        return Padding(
          padding: const EdgeInsets.all(16),
          child: ListView(
            children: [
              Text('Backend (texture_tool)', style: Theme.of(context).textTheme.headlineSmall),
              const SizedBox(height: 12),
              Text(
                'How the app picks the executable (first match wins):',
                style: Theme.of(context).textTheme.titleSmall,
              ),
              const SizedBox(height: 8),
              Text(
                '• Path saved below (this device), if the file still exists.\n'
                '• Bundled copy: texture_tool next to the app binary in the .app (from '
                'macOS build phase if backend/target/release or debug was built first).\n'
                '• Environment variable TEXTURE_TOOL_PATH, if set and the file exists.\n'
                '• Development: ../backend/target/release/texture_tool then '
                '../backend/target/debug/texture_tool relative to the process '
                'current working directory (often the frontend folder when using flutter run).\n'
                '• A program named texture_tool on your PATH (which / where).\n\n'
                'For a built macOS .app, the bundled copy is used when present; otherwise '
                'use Choose executable… or TEXTURE_TOOL_PATH.',
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              const SizedBox(height: 16),
              InputDecorator(
                decoration: const InputDecoration(
                  labelText: 'Saved executable path (optional)',
                  border: OutlineInputBorder(),
                ),
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: SelectableText(
                    widget.appState.textureToolExecutableOverride ??
                        'None — automatic resolution',
                  ),
                ),
              ),
              const SizedBox(height: 8),
              Text(
                resolvedExe != null
                    ? 'Effective executable for scans: $resolvedExe'
                    : 'No executable resolved yet. Add a path above or satisfy a later step.',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  FilledButton.icon(
                    onPressed: _pickExecutable,
                    icon: const Icon(Icons.folder_open),
                    label: const Text('Choose executable…'),
                  ),
                  OutlinedButton(
                    onPressed: widget.appState.textureToolExecutableOverride == null
                        ? null
                        : _clearExecutableOverride,
                    child: const Text('Clear override'),
                  ),
                ],
              ),
              const Divider(height: 40),
              Text('Analysis settings', style: Theme.of(context).textTheme.headlineSmall),
              const SizedBox(height: 8),
              SwitchListTile(
                title: const Text('Enable pHash'),
                value: _cfg.enablePhash,
                onChanged: (v) => setState(() => _cfg.enablePhash = v),
              ),
              SwitchListTile(
                title: const Text('Enable SSIM'),
                value: _cfg.enableSsim,
                onChanged: (v) => setState(() => _cfg.enableSsim = v),
              ),
              SwitchListTile(
                title: const Text('Enable histogram'),
                value: _cfg.enableHistogram,
                onChanged: (v) => setState(() => _cfg.enableHistogram = v),
              ),
              SwitchListTile(
                title: const Text('Alpha crop'),
                subtitle: const Text('Crop to alpha bbox before metrics'),
                value: _cfg.enableAlphaCrop,
                onChanged: (v) => setState(() => _cfg.enableAlphaCrop = v),
              ),
              SwitchListTile(
                title: const Text('Rotations (max over B)'),
                value: _cfg.enableRotations,
                onChanged: (v) => setState(() => _cfg.enableRotations = v),
              ),
              SwitchListTile(
                title: const Text('Flip (with rotations)'),
                value: _cfg.enableFlip,
                onChanged: (v) => setState(() => _cfg.enableFlip = v),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: _thr,
                decoration: const InputDecoration(
                  labelText: 'Clustering threshold (final_score)',
                  hintText: '0.0 – 1.0 (edge if score > threshold)',
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                'Weights (need not sum to 1; backend renormalizes per pair)',
              ),
              TextField(
                controller: _wPhash,
                decoration: const InputDecoration(labelText: 'Weight pHash'),
              ),
              TextField(
                controller: _wSsim,
                decoration: const InputDecoration(labelText: 'Weight SSIM'),
              ),
              TextField(
                controller: _wHist,
                decoration: const InputDecoration(labelText: 'Weight histogram'),
              ),
              const Divider(height: 32),
              Text('Advanced', style: Theme.of(context).textTheme.titleMedium),
              TextField(
                controller: _hashAlg,
                decoration: const InputDecoration(
                  labelText: 'hash_algorithm',
                  hintText: 'sha256 | md5 | sha1',
                ),
              ),
              TextField(
                controller: _phashDist,
                decoration: const InputDecoration(labelText: 'phash_max_distance'),
                keyboardType: TextInputType.number,
              ),
              TextField(
                controller: _ssimThr,
                decoration: const InputDecoration(
                  labelText: 'ssim_threshold (gate)',
                ),
              ),
              TextField(
                controller: _resize,
                decoration: const InputDecoration(labelText: 'resize_size'),
                keyboardType: TextInputType.number,
              ),
              TextField(
                controller: _histBins,
                decoration: const InputDecoration(labelText: 'hist_bins'),
                keyboardType: TextInputType.number,
              ),
              TextField(
                controller: _histMethod,
                decoration: const InputDecoration(
                  labelText: 'hist_method',
                  hintText: 'correlation | bhattacharyya',
                ),
              ),
              TextField(
                controller: _alphaThr,
                decoration: const InputDecoration(labelText: 'alpha_threshold'),
              ),
              TextField(
                controller: _maxDecode,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'max_decode_dimension_px (empty = unlimited)',
                ),
              ),
              const SizedBox(height: 24),
              Row(
                children: [
                  FilledButton(onPressed: _apply, child: const Text('Save settings')),
                  const SizedBox(width: 12),
                  OutlinedButton(onPressed: _resetDefaults, child: const Text('Reset defaults')),
                ],
              ),
              const SizedBox(height: 16),
              OutlinedButton.icon(
                onPressed: () async {
                  try {
                    final f = await writeConfigToTempFile(widget.appState.config);
                    widget.appState.setLastOutputs(configPath: f.path);
                    if (!context.mounted) return;
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Wrote ${f.path}')),
                    );
                  } catch (e) {
                    if (!context.mounted) return;
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Write failed: $e')),
                    );
                  }
                },
                icon: const Icon(Icons.save_alt),
                label: const Text('Export config to temp file'),
              ),
            ],
          ),
        );
      },
    );
  }
}
