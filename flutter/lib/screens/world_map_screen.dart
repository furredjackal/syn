import 'package:flutter/material.dart';

class District {
  final String name;
  final String description;
  final IconData icon;
  final Color color;
  int npcCount;

  District({
    required this.name,
    required this.description,
    required this.icon,
    required this.color,
    this.npcCount = 0,
  });
}

class WorldMapScreen extends StatefulWidget {
  const WorldMapScreen({Key? key}) : super(key: key);

  @override
  State<WorldMapScreen> createState() => _WorldMapScreenState();
}

class _WorldMapScreenState extends State<WorldMapScreen> {
  late List<District> districts;
  District? selectedDistrict;

  @override
  void initState() {
    super.initState();
    districts = [
      District(
        name: 'DOWNTOWN',
        description: 'Business hub, restaurants, entertainment',
        icon: Icons.business,
        color: Colors.blue,
        npcCount: 12,
      ),
      District(
        name: 'RESIDENTIAL',
        description: 'Homes, neighborhoods, parks',
        icon: Icons.home,
        color: Colors.green,
        npcCount: 8,
      ),
      District(
        name: 'INDUSTRIAL',
        description: 'Factories, warehouses, offices',
        icon: Icons.factory,
        color: Colors.grey,
        npcCount: 5,
      ),
      District(
        name: 'WATERFRONT',
        description: 'Docks, beaches, entertainment',
        icon: Icons.water,
        color: Colors.cyan,
        npcCount: 6,
      ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text(
          'WORLD MAP',
          style: Theme.of(context)
              .textTheme
              .titleMedium
              ?.copyWith(color: const Color(0xFF00D9FF)),
        ),
        leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: () => Navigator.pop(context)),
      ),
      body: Column(
        children: [
          Expanded(
            child: GridView.builder(
              padding: const EdgeInsets.all(16),
              gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: 2,
                childAspectRatio: 1,
                crossAxisSpacing: 12,
                mainAxisSpacing: 12,
              ),
              itemCount: districts.length,
              itemBuilder: (context, index) =>
                  _buildDistrictCard(context, districts[index]),
            ),
          ),
          if (selectedDistrict != null) ...[
            Divider(color: Colors.white10),
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    selectedDistrict!.name,
                    style: Theme.of(context)
                        .textTheme
                        .titleMedium
                        ?.copyWith(color: selectedDistrict!.color),
                  ),
                  const SizedBox(height: 8),
                  Text(selectedDistrict!.description,
                      style: Theme.of(context).textTheme.bodySmall),
                  const SizedBox(height: 12),
                  ElevatedButton(
                    onPressed: () {},
                    child: const Text('VISIT DISTRICT'),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildDistrictCard(BuildContext context, District district) {
    final isSelected = selectedDistrict == district;
    return GestureDetector(
      onTap: () => setState(() => selectedDistrict = district),
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(
            color: isSelected ? district.color : Colors.white10,
            width: isSelected ? 2 : 1,
          ),
          borderRadius: BorderRadius.circular(4),
          color: Colors.black.withOpacity(0.3),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(district.icon, color: district.color, size: 48),
            const SizedBox(height: 12),
            Text(district.name,
                style: Theme.of(context)
                    .textTheme
                    .bodySmall
                    ?.copyWith(color: district.color)),
            Text('${district.npcCount} residents',
                style: Theme.of(context).textTheme.labelSmall),
          ],
        ),
      ),
    );
  }
}
