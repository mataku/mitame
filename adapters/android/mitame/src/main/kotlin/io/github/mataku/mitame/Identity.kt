package io.github.mataku.mitame

internal object Identity {
    private val camelBoundary = Regex("([a-z0-9])([A-Z])")
    private val acronymBoundary = Regex("([A-Z])([A-Z][a-z])")
    private val unsafe = Regex("[^a-z0-9_.-]")

    fun normalize(raw: String): String {
        var value = raw.replace(camelBoundary, "$1_$2").replace(acronymBoundary, "$1_$2").lowercase().replace(unsafe, "_")
        while (value.contains("__")) {
            value = value.replace("__", "_")
        }
        value = value.trimStart('.', '_')
        return value.ifEmpty { "_" }
    }

    fun encodeVariant(variant: Map<String, String>): String =
        variant.entries.sortedBy { it.key }.joinToString(",") { "${it.key}=${it.value}" }

    fun stem(name: String, variant: Map<String, String>): String =
        if (variant.isEmpty()) name else "${name}__${encodeVariant(variant)}"

    fun id(platform: String, group: List<String>, stem: String): String =
        (listOf(platform) + group + stem).joinToString("/")
}
