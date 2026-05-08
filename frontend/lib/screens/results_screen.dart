import 'dart:io';

import 'package:flutter/material.dart';

import '../app_state.dart';

class ResultsScreen extends StatefulWidget {
  const ResultsScreen({super.key, required this.appState});

  final AppState appState;

  @override
  State<ResultsScreen> createState() => _ResultsScreenState();
}

class _ResultsScreenState extends State<ResultsScreen> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.appState,
      builder: (context, _) {
        final data = widget.appState.lastScanResult;
        final path = widget.appState.lastResultPath;

        if (data == null || data.groups.isEmpty) {
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

        final groups = data.groups;
        if (_selectedIndex >= groups.length) {
          _selectedIndex = 0;
        }
        final selected = groups[_selectedIndex];

        return Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(
              width: 280,
              child: Material(
                elevation: 1,
                child: ListView.builder(
                  itemCount: groups.length,
                  itemBuilder: (context, i) {
                    final g = groups[i];
                    final subtitle = g.score != null
                        ? 'score: ${g.score!.toStringAsFixed(3)} · ${g.count} images'
                        : '${g.count} images';
                    return ListTile(
                      selected: i == _selectedIndex,
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
                              : () => _revealInFileManager(selected.images.first),
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
      return _placeholder(Icons.broken_image, 'Missing');
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: Image.file(
        file,
        fit: BoxFit.cover,
        errorBuilder: (context, error, stackTrace) =>
            _placeholder(Icons.error_outline, 'Error'),
        frameBuilder: (ctx, child, frame, w) {
          if (frame == null) {
            return const Center(child: CircularProgressIndicator(strokeWidth: 2));
          }
          return child;
        },
      ),
    );
  }

  Widget _placeholder(IconData icon, String label) {
    return DecoratedBox(
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.shade400),
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
