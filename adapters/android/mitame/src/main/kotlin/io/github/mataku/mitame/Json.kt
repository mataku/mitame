package io.github.mataku.mitame

internal object Json {
    fun encode(value: Any?, indent: String = ""): String = when (value) {
        null -> "null"
        is String -> quote(value)
        is Boolean, is Int, is Long -> value.toString()
        is Float -> encodeNumber(value.toDouble())
        is Double -> encodeNumber(value)
        is Map<*, *> -> encodeObject(value, indent)
        is List<*> -> encodeList(value, indent)
        else -> quote(value.toString())
    }

    private fun encodeNumber(value: Double): String =
        if (value == Math.floor(value) && !value.isInfinite()) "%.1f".format(java.util.Locale.ROOT, value) else value.toString()

    private fun encodeObject(map: Map<*, *>, indent: String): String {
        if (map.isEmpty()) return "{}"
        val inner = "$indent  "
        return map.entries.joinToString(",\n", "{\n", "\n$indent}") { (k, v) ->
            "$inner${quote(k.toString())}: ${encode(v, inner)}"
        }
    }

    private fun encodeList(list: List<*>, indent: String): String {
        if (list.isEmpty()) return "[]"
        val inner = "$indent  "
        return list.joinToString(",\n", "[\n", "\n$indent]") { "$inner${encode(it, inner)}" }
    }

    private fun quote(s: String): String {
        val sb = StringBuilder("\"")
        for (c in s) {
            when (c) {
                '"' -> sb.append("\\\"")
                '\\' -> sb.append("\\\\")
                '\n' -> sb.append("\\n")
                '\r' -> sb.append("\\r")
                '\t' -> sb.append("\\t")
                else -> if (c < ' ') sb.append(String.format("\\u%04x", c.code)) else sb.append(c)
            }
        }
        return sb.append('"').toString()
    }

    fun readRunId(text: String): String? {
        val match = Regex("\"run_id\"\\s*:\\s*\"([^\"]*)\"").find(text)
        return match?.groupValues?.get(1)
    }
}
