import 'dart:convert';

import 'dart:io';

/// Parsed [result.json](Tasks/Task.md §6).
class ScanResult {
  ScanResult({required this.groups});

  final List<ResultGroup> groups;

  factory ScanResult.fromJson(Map<String, dynamic> json) {
    final raw = json['groups'];
    if (raw is! List) {
      throw const FormatException('result.json: missing or invalid "groups" array');
    }
    return ScanResult(
      groups: raw
          .map((e) => ResultGroup.fromJson(e as Map<String, dynamic>))
          .toList(),
    );
  }

  static ScanResult parseFile(String path) {
    final text = File(path).readAsStringSync();
    return parseString(text);
  }

  static Future<ScanResult> parseFileAsync(String path) async {
    final text = await File(path).readAsString();
    return parseString(text);
  }

  static ScanResult parseString(String jsonText) {
    final decoded = jsonDecode(jsonText);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('result.json: expected top-level object');
    }
    return ScanResult.fromJson(decoded);
  }
}

class ResultGroup {
  ResultGroup({
    required this.id,
    required this.images,
    this.score,
  });

  final int id;
  final double? score;
  final List<String> images;

  factory ResultGroup.fromJson(Map<String, dynamic> json) {
    final idRaw = json['id'];
    final int id;
    if (idRaw is int) {
      id = idRaw;
    } else if (idRaw is double) {
      id = idRaw.toInt();
    } else {
      throw const FormatException('group: missing id');
    }
    final imgs = json['images'];
    if (imgs is! List) {
      throw const FormatException('group: missing images');
    }
    final paths = imgs.map((e) => e.toString()).toList();
    final scoreRaw = json['score'];
    double? score;
    if (scoreRaw != null) {
      if (scoreRaw is num) {
        score = scoreRaw.toDouble();
      } else {
        throw const FormatException('group: invalid score');
      }
    }
    return ResultGroup(id: id, images: paths, score: score);
  }

  int get count => images.length;
}
