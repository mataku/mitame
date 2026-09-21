final RegExp _safeComponent = RegExp(r'^[a-z0-9_.-]+$');
final RegExp _unsafeChars = RegExp(r'[^a-z0-9_.-]');

String normalizeComponent(String raw) {
  var value = raw.toLowerCase().replaceAll(_unsafeChars, '_');
  while (value.contains('__')) {
    value = value.replaceAll('__', '_');
  }
  value = value.replaceAll(RegExp(r'^[._]+'), '');
  if (value.isEmpty) {
    value = '_';
  }
  return value;
}

bool isSafeComponent(String value) =>
    _safeComponent.hasMatch(value) &&
    !value.startsWith('.') &&
    !value.contains('__');

String encodeVariant(Map<String, String> variant) {
  final keys = variant.keys.toList()..sort();
  return keys.map((k) => '$k=${variant[k]}').join(',');
}

Map<String, String> decodeVariant(String encoded) {
  if (encoded.isEmpty) {
    return const {};
  }
  final result = <String, String>{};
  for (final pair in encoded.split(',')) {
    final index = pair.indexOf('=');
    if (index <= 0) {
      throw FormatException('Invalid variant pair: $pair');
    }
    result[pair.substring(0, index)] = pair.substring(index + 1);
  }
  return result;
}

class MitameIdentity {
  MitameIdentity({
    required this.platform,
    required this.group,
    required this.name,
    Map<String, String> variant = const {},
  }) : variant = Map.unmodifiable(variant) {
    for (final component in [platform, ...group, name]) {
      if (!isSafeComponent(component)) {
        throw ArgumentError('Unsafe identity component: $component');
      }
    }
    for (final entry in variant.entries) {
      if (!isSafeComponent(entry.key) || !isSafeComponent(entry.value)) {
        throw ArgumentError('Unsafe variant: ${entry.key}=${entry.value}');
      }
    }
  }

  final String platform;
  final List<String> group;
  final String name;
  final Map<String, String> variant;

  String get stem =>
      variant.isEmpty ? name : '${name}__${encodeVariant(variant)}';

  String get id => [platform, ...group, stem].join('/');

  String get relativePath => '$id.png';

  String get sidecarRelativePath => '$id.json';
}
