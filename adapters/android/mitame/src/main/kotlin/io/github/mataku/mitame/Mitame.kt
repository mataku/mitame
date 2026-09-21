package io.github.mataku.mitame

import android.graphics.Bitmap
import android.graphics.Canvas
import android.os.Build
import android.view.View
import java.io.File
import java.time.Instant

object Mitame {
    const val OUTPUT_DIR_ENV = "MITAME_OUTPUT_DIR"
    const val PROFILE_ENV = "MITAME_PROFILE"
    const val RUN_ID_ENV = "MITAME_RUN_ID"
    const val PLATFORM = "android"
    const val SCHEMA_VERSION = 1

    private const val ADAPTER_PACKAGE = "io.github.mataku.mitame"

    private val FRAMEWORK_PREFIXES = listOf(
        "java.", "javax.", "jdk.", "sun.", "kotlin.", "kotlinx.",
        "org.junit.", "org.robolectric.", "android.", "androidx.", "dalvik.",
    )

    fun capture(
        view: View,
        name: String,
        variant: Map<String, String> = emptyMap(),
        group: String? = null,
        widthPx: Int? = null,
        heightPx: Int? = null,
    ): File {
        val width = widthPx ?: view.width.takeIf { it > 0 } ?: view.measuredWidth
        val height = heightPx ?: view.height.takeIf { it > 0 } ?: view.measuredHeight
        require(width > 0 && height > 0) { "view has no size; pass widthPx and heightPx or lay it out first" }
        if (view.width != width || view.height != height) {
            view.measure(
                View.MeasureSpec.makeMeasureSpec(width, View.MeasureSpec.EXACTLY),
                View.MeasureSpec.makeMeasureSpec(height, View.MeasureSpec.EXACTLY),
            )
            view.layout(0, 0, width, height)
        }
        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        view.draw(Canvas(bitmap))
        val scale = view.resources.displayMetrics.density
        return write(bitmap, name, variant, group ?: callerGroup(), scale, displayName = name)
    }

    fun write(
        bitmap: Bitmap,
        name: String,
        variant: Map<String, String> = emptyMap(),
        group: String? = null,
        scale: Float = 1f,
        displayName: String? = null,
    ): File {
        val env = System.getenv()
        val outputDir = File(env[OUTPUT_DIR_ENV] ?: File(System.getProperty("user.dir"), ".mitame/current").path)
        val profile = env[PROFILE_ENV] ?: "default"
        val runId = env[RUN_ID_ENV]
        val groupSegments = (group ?: callerGroup()).split('/').filter { it.isNotEmpty() }.map(Identity::normalize)
        val normalizedName = Identity.normalize(name)
        val normalizedVariant = variant.entries.associate { Identity.normalize(it.key) to Identity.normalize(it.value) }
        val stem = Identity.stem(normalizedName, normalizedVariant)
        val id = Identity.id(PLATFORM, groupSegments, stem)
        val root = File(outputDir, profile)
        val png = File(root, "$id.png")
        val sidecar = File(root, "$id.json")
        checkCollision(id, sidecar, runId)
        png.parentFile?.mkdirs()
        png.outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
        val fingerprint = Build.FINGERPRINT ?: ""
        val document = linkedMapOf<String, Any?>(
            "schema_version" to SCHEMA_VERSION,
            "id" to id,
            "platform" to PLATFORM,
            "capture" to "widget",
            "group" to groupSegments.joinToString("/"),
            "name" to normalizedName,
            "display_name" to (displayName ?: name),
            "variant" to normalizedVariant.toSortedMap(),
            "image" to linkedMapOf("width" to bitmap.width, "height" to bitmap.height, "scale" to scale.toDouble()),
            "env" to linkedMapOf<String, Any?>(
                "os" to "android",
                "sdk" to Build.VERSION.SDK_INT.toString(),
                "device" to Build.MODEL,
                "renderer" to if (fingerprint.contains("robolectric")) "robolectric" else null,
                "ci" to env.containsKey("CI"),
            ).filterValues { it != null },
            "captured_at" to Instant.now().toString(),
            "ext" to linkedMapOf("android" to linkedMapOf<String, Any?>("run_id" to runId).filterValues { it != null }),
        )
        sidecar.writeText(Json.encode(document) + "\n")
        return png
    }

    private fun checkCollision(id: String, sidecar: File, runId: String?) {
        if (runId == null || !sidecar.exists()) return
        if (Json.readRunId(sidecar.readText()) == runId) {
            throw IllegalStateException(
                "Two captures resolved to the same mitame identity \"$id\" in this run. Use distinct names or pass group explicitly.",
            )
        }
    }

    private fun callerGroup(): String {
        val frame = Throwable().stackTrace.firstOrNull { frame ->
            val cls = frame.className
            cls.substringBeforeLast('.') != ADAPTER_PACKAGE && FRAMEWORK_PREFIXES.none { cls.startsWith(it) }
        } ?: return "_"
        return frame.className.substringAfterLast('.').substringBefore('$')
    }
}
