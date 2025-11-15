import 'package:flutter/material.dart';

class HeatmapWidget extends StatelessWidget {
  final List<String> rowLabels;
  final List<String> columnLabels;
  final List<List<double>> data; // values from -10 to 10
  final double cellSize;

  const HeatmapWidget({
    Key? key,
    required this.rowLabels,
    required this.columnLabels,
    required this.data,
    this.cellSize = 60,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Column headers
        Padding(
          padding: const EdgeInsets.only(left: 120),
          child: Row(
            children: columnLabels.map((label) {
              return SizedBox(
                width: cellSize,
                child: Center(
                  child: Text(
                    label,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.cyan,
                        ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              );
            }).toList(),
          ),
        ),
        const SizedBox(height: 8),
        // Heatmap rows
        ...List.generate(rowLabels.length, (rowIndex) {
          return Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Row(
              children: [
                SizedBox(
                  width: 120,
                  child: Text(
                    rowLabels[rowIndex],
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.grey[400],
                        ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                ...List.generate(columnLabels.length, (colIndex) {
                  final value = data[rowIndex][colIndex];
                  final normalized = (value + 10) / 20; // Convert -10..10 to 0..1
                  final color = _getHeatmapColor(normalized);

                  return Tooltip(
                    message: '${rowLabels[rowIndex]} → ${columnLabels[colIndex]}: ${value.toStringAsFixed(1)}',
                    child: MouseRegion(
                      cursor: SystemMouseCursors.click,
                      child: Container(
                        width: cellSize,
                        height: cellSize,
                        margin: const EdgeInsets.symmetric(horizontal: 2),
                        decoration: BoxDecoration(
                          color: color,
                          border: Border.all(color: Colors.grey.shade800, width: 1),
                          borderRadius: BorderRadius.circular(2),
                        ),
                        child: Center(
                          child: Text(
                            value.toStringAsFixed(1),
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 10,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ),
                    ),
                  );
                }).toList(),
              ],
            ),
          );
        }),
      ],
    );
  }

  Color _getHeatmapColor(double normalized) {
    if (normalized < 0.25) {
      return Color.lerp(Colors.red, Colors.orange, normalized * 4)!;
    } else if (normalized < 0.5) {
      return Color.lerp(Colors.orange, Colors.yellow, (normalized - 0.25) * 4)!;
    } else if (normalized < 0.75) {
      return Color.lerp(Colors.yellow, Colors.cyan, (normalized - 0.5) * 4)!;
    } else {
      return Color.lerp(Colors.cyan, Colors.green, (normalized - 0.75) * 4)!;
    }
  }
}
