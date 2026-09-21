package io.github.mataku.mitame.compose

import androidx.compose.ui.graphics.asAndroidBitmap
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.junit4.ComposeTestRule
import androidx.compose.ui.test.onRoot
import io.github.mataku.mitame.Mitame
import java.io.File

fun SemanticsNodeInteraction.captureMitame(
    name: String,
    variant: Map<String, String> = emptyMap(),
    group: String? = null,
    scale: Float? = null,
): File {
    val node = fetchSemanticsNode()
    val density = scale ?: node.layoutInfo.density.density
    val bitmap = captureToImage().asAndroidBitmap()
    return Mitame.write(bitmap, name, variant, group ?: Mitame.callerGroup(), density, displayName = name)
}

fun ComposeTestRule.captureMitame(
    name: String,
    variant: Map<String, String> = emptyMap(),
    group: String? = null,
    scale: Float? = null,
): File = onRoot().captureMitame(name, variant, group ?: Mitame.callerGroup(), scale)
