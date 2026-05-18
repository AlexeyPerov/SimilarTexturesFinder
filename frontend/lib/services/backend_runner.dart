import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

Process? _activeTextureToolProcess;

/// Terminates a running `texture_tool` child, if any (e.g. app quit during scan).
void killActiveTextureTool() {
  final p0 = _activeTextureToolProcess;
  if (p0 == null) return;
  try {
    p0.kill(ProcessSignal.sigkill);
  } catch (_) {}
  _activeTextureToolProcess = null;
}

/// Result of spawning `texture_tool`.
class BackendRunResult {
  BackendRunResult({
    required this.exitCode,
    required this.stdoutText,
    required this.stderrText,
  });

  final int exitCode;
  final String stdoutText;
  final String stderrText;

  bool get success => exitCode == 0;

  /// Last ~40 lines of stderr for dialogs.
  String get stderrTail {
    final lines = stderrText.split('\n');
    if (lines.length <= 40) return stderrText;
    return lines.sublist(lines.length - 40).join('\n');
  }
}

/// Resolve `texture_tool` binary: app override, bundled (macOS), env, dev, PATH.
String? resolveTextureToolExecutable({String? override}) {
  if (override != null && override.isNotEmpty) {
    final o = File(override);
    if (o.existsSync()) return override;
  }

  final bundled = _bundledTextureToolPath();
  if (bundled != null) return bundled;

  final env = Platform.environment['TEXTURE_TOOL_PATH'];
  if (env != null && env.isNotEmpty) {
    final f = File(env);
    if (f.existsSync()) return env;
  }

  final name = Platform.isWindows ? 'texture_tool.exe' : 'texture_tool';
  final cwd = Directory.current.path;

  final release = p.join(cwd, '..', 'backend', 'target', 'release', name);
  if (File(release).existsSync()) return p.normalize(release);

  final debug = p.join(cwd, '..', 'backend', 'target', 'debug', name);
  if (File(debug).existsSync()) return p.normalize(debug);

  final pathHit = _whichTextureTool(name);
  if (pathHit != null) return pathHit;

  return null;
}

/// Next to the Flutter/Dart executable inside `.app/Contents/MacOS/` (Xcode copy phase).
String? _bundledTextureToolPath() {
  if (Platform.isMacOS) {
    try {
      final self = File(Platform.resolvedExecutable);
      final sibling = File(p.join(self.parent.path, 'texture_tool'));
      if (sibling.existsSync()) {
        return p.normalize(sibling.path);
      }
    } catch (_) {}
  }
  return null;
}

String? _whichTextureTool(String fileName) {
  try {
    if (Platform.isWindows) {
      final r = Process.runSync('where', [fileName]);
      if (r.exitCode != 0) return null;
      final line = (r.stdout as String).trim().split(RegExp(r'\r?\n')).first.trim();
      if (line.isEmpty) return null;
      return File(line).existsSync() ? line : null;
    } else {
      final base = fileName.replaceAll('.exe', '');
      final r = Process.runSync('which', [base]);
      if (r.exitCode != 0) return null;
      final line = (r.stdout as String).trim();
      if (line.isEmpty) return null;
      return File(line).existsSync() ? line : null;
    }
  } catch (_) {
    return null;
  }
}

Future<BackendRunResult> runTextureTool({
  required String executable,
  required String inputDir,
  required String outputJson,
  required String configPath,
  int threads = 4,
  void Function(String chunk, {required bool isStderr})? onOutput,
}) async {
  final process = await Process.start(executable, [
    '--input',
    inputDir,
    '--output',
    outputJson,
    '--config',
    configPath,
    '--threads',
    '$threads',
  ]);

  final stdoutBuf = StringBuffer();
  final stderrBuf = StringBuffer();

  Future<void> drain(
    Stream<List<int>> stream,
    StringBuffer acc,
    bool isStderr,
  ) async {
    try {
      await for (final chunk in stream.transform(utf8.decoder)) {
        acc.write(chunk);
        onOutput?.call(chunk, isStderr: isStderr);
      }
    } catch (_) {
      // Broken pipe / process killed — partial output is still useful.
    }
  }

  late final int exitCode;
  try {
    _activeTextureToolProcess = process;
    await Future.wait<void>([
      drain(process.stdout, stdoutBuf, false),
      drain(process.stderr, stderrBuf, true),
      process.exitCode.then((c) {
        exitCode = c;
      }),
    ]);
  } finally {
    if (_activeTextureToolProcess == process) {
      _activeTextureToolProcess = null;
    }
  }

  return BackendRunResult(
    exitCode: exitCode,
    stdoutText: stdoutBuf.toString(),
    stderrText: stderrBuf.toString(),
  );
}
