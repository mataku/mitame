package io.github.mataku.mitame.example

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.util.TypedValue
import android.view.Gravity
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView

class LoginFormView(context: Context, dark: Boolean = false, title: String = "Sign in") : LinearLayout(context) {
    init {
        orientation = VERTICAL
        val pad = dp(16)
        setPadding(pad, pad, pad, pad)
        setBackgroundColor(if (dark) Color.parseColor("#141218") else Color.parseColor("#FEF7FF"))
        val fg = if (dark) Color.parseColor("#E6E0E9") else Color.parseColor("#1D1B20")

        addView(TextView(context).apply {
            text = title
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 24f)
            setTextColor(fg)
            typeface = Typeface.DEFAULT_BOLD
        })
        addView(EditText(context).apply {
            hint = "Email"
            setTextColor(fg)
            setHintTextColor(fg and 0x99FFFFFF.toInt())
        }, LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT).apply { topMargin = dp(16) })
        addView(EditText(context).apply {
            hint = "Password"
            setTextColor(fg)
            setHintTextColor(fg and 0x99FFFFFF.toInt())
        }, LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT).apply { topMargin = dp(8) })
        addView(Button(context).apply {
            text = "Continue"
            gravity = Gravity.CENTER
            setBackgroundColor(Color.parseColor("#6750A4"))
            setTextColor(Color.WHITE)
        }, LayoutParams(LayoutParams.MATCH_PARENT, dp(48)).apply { topMargin = dp(16) })
    }

    private fun dp(value: Int): Int =
        TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, value.toFloat(), resources.displayMetrics).toInt()
}
