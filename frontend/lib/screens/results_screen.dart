import 'dart:io';

import 'package:flutter/material.dart';

import '../app_state.dart';
import '../services/json_parser.dart';

class ResultsScreen extends StatefulWidget {
  const ResultsScreen({super.key, required this.appState});

  final AppState appState;

  @override
  State<ResultsScreen> createState() => _ResultsScreenState();
}

class _ResultsScreenState extends State<ResultsScreen> {
  int _selectedIndex = 0;

  @override
  void didUpdateWidget(ResultsScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.appState.hideSingletonResultGroups !=
        widget.appState.hideSingletonResultGroups) {
      _selectedIndex = 0;
    }
  }

  List<ResultGroup> _visibleGroups(List<ResultGroup> raw, bool hideSingletons) {
    if (!hideSingletons) return raw;
    return raw.where((g) => g.images.length >= 2).toList();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.appState,
      builder: (context, _) {
        final hideSingletons = widget.appState.hideSingletonResultGroups;
        final data = widget.appState.lastScanResult;
        final path = widget.appState.lastResultPath;

        if (data == null) {
          return Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Text(
                path == null
                    ? 'No results yet. Run a scan from the Scan tab.'
                    : 'Could not load results. Path: $path',
                textAlign: TextAlign.center,
              ),
            ),
          );
        }

        final visible = _visibleGroups(data.groups, hideSingletons);
        if (visible.isEmpty) {
          return const Center(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Text(
                'No issues found.',
                textAlign: TextAlign.center,
              ),
            ),
          );
        }

        final displayIndex =
            _selectedIndex.clamp(0, visible.length - 1);
        if (displayIndex != _selectedIndex) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (mounted) {
              setState(() => _selectedIndex = displayIndex);
            }
          });
        }
        final selected = visible[displayIndex];

        return Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(
              width: 280,
              child: Material(
                elevation: 1,
                child: ListView.builder(
                  itemCount: visible.length,
                  itemBuilder: (context, i) {
                    final g = visible[i];
                    final subtitle = g.score != null
                        ? 'score: ${g.score!.toStringAsFixed(3)} · ${g.count} images'
                        : '${g.count} images';
                    return ListTile(
                      selected: i == displayIndex,
                      title: Text('Group ${g.id}'),
                      subtitle: Text(subtitle),
                      onTap: () => setState(() => _selectedIndex = i),
                    );
                  },
                ),
              ),
            ),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Row(
                      children: [
                        Text(
                          'Group ${selected.id}',
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                        const SizedBox(width: 16),
                        if (selected.score != null)
                          Text(
                            'Score: ${selected.score!.toStringAsFixed(3)}',
                            style: Theme.of(context).textTheme.titleMedium,
                          )
                        else
                          Text(
                            'Score: — (singleton or undefined)',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                        const Spacer(),
                        FilledButton.tonalIcon(
                          onPressed: selected.images.isEmpty
                              ? null
                              : () =>
                                  _revealInFileManager(selected.images.first),
                          icon: const Icon(Icons.folder_open),
                          label: const Text('Reveal in Finder / explorer'),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Expanded(
                      child: GridView.builder(
                        gridDelegate:
                            const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: 4,
                          mainAxisSpacing: 8,
                          crossAxisSpacing: 8,
                          childAspectRatio: 1,
                        ),
                        itemCount: selected.images.length,
                        itemBuilder: (context, index) {
                          final imgPath = selected.images[index];
                          return _ThumbTile(path: imgPath);
                        },
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  Future<void> _revealInFileManager(String imagePath) async {
    try {
      if (Platform.isMacOS) {
        await Process.run('open', ['-R', imagePath]);
      } else if (Platform.isWindows) {
        await Process.run('explorer', ['/select,', imagePath]);
      } else {
        await Process.run('xdg-open', [File(imagePath).parent.path]);
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Could not open file manager: $e')),
        );
      }
    }
  }
}

class _ThumbTile extends StatelessWidget {
  const _ThumbTile({required this.path});

  final String path;

  @override
  Widget build(BuildContext context) {
    final file = File(path);
    if (!file.existsSync()) {
      return _placeholder(context, Icons.broken_image, 'Missing');
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: Image.file(
        file,
        fit: BoxFit.cover,
        cacheWidth: 256,
        cacheHeight: 256,
        errorBuilder: (context, error, stackTrace) =>
            _placeholder(context, Icons.error_outline, 'Error'),
        frameBuilder: (ctx, child, frame, w) {
          if (frame == null) {
            return const Center(child: CircularProgressIndicator(strokeWidth: 2));
          }
          return child;
        },
      ),
    );
  }

  Widget _placeholder(BuildContext context, IconData icon, String label) {
    return DecoratedBox(
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(icon),
          Text(label, textAlign: TextAlign.center),
        ],
      ),
    );
  }
}
