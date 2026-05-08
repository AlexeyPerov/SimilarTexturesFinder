import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

const _prefsFileName = 'user_preferences.json';
const _keyTextureToolExecutable = 'texture_tool_executable';
const _keyScanInputFolder = 'scan_input_folder';

/// Loads and saves app-wide user preferences (JSON under application support).
class UserPreferencesStore {
  UserPreferencesStore({File? fileForTest})
      : _fileOverride = fileForTest;

  final File? _fileOverride;

  Future<File> _prefsFile() async {
    if (_fileOverride != null) return _fileOverride;
    final dir = await getApplicationSupportDirectory();
    return File(p.join(dir.path, _prefsFileName));
  }

  Future<String?> loadTextureToolExecutableOverride() async {
    try {
      final file = await _prefsFile();
      if (!file.existsSync()) return null;
      final map = jsonDecode(file.readAsStringSync());
      if (map is! Map<String, dynamic>) return null;
      final v = map[_keyTextureToolExecutable];
      if (v is! String || v.isEmpty) return null;
      return v;
    } catch (_) {
      return null;
    }
  }

  Future<void> saveTextureToolExecutableOverride(String? path) async {
    final file = await _prefsFile();
    Map<String, dynamic> map = {};
    if (file.existsSync()) {
      try {
        final decoded = jsonDecode(file.readAsStringSync());
        if (decoded is Map<String, dynamic>) {
          map = Map<String, dynamic>.from(decoded);
        }
      } catch (_) {}
    }
    if (path == null || path.isEmpty) {
      map.remove(_keyTextureToolExecutable);
    } else {
      map[_keyTextureToolExecutable] = path;
    }
    await file.parent.create(recursive: true);
    await file.writeAsString('${const JsonEncoder.withIndent('  ').convert(map)}\n');
  }

  Future<String?> loadLastScanInputFolder() async {
    try {
      final file = await _prefsFile();
      if (!file.existsSync()) return null;
      final map = jsonDecode(file.readAsStringSync());
      if (map is! Map<String, dynamic>) return null;
      final v = map[_keyScanInputFolder];
      if (v is! String || v.isEmpty) return null;
      return v;
    } catch (_) {
      return null;
    }
  }

  Future<void> saveLastScanInputFolder(String? path) async {
    final file = await _prefsFile();
    Map<String, dynamic> map = {};
    if (file.existsSync()) {
      try {
        final decoded = jsonDecode(file.readAsStringSync());
        if (decoded is Map<String, dynamic>) {
          map = Map<String, dynamic>.from(decoded);
        }
      } catch (_) {}
    }
    if (path == null || path.isEmpty) {
      map.remove(_keyScanInputFolder);
    } else {
      map[_keyScanInputFolder] = path;
    }
    await file.parent.create(recursive: true);
    await file.writeAsString('${const JsonEncoder.withIndent('  ').convert(map)}\n');
  }
}
