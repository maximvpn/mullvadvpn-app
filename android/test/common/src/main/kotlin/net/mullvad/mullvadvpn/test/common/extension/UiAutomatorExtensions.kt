package net.mullvad.mullvadvpn.test.common.extension

import android.os.Build
import androidx.test.uiautomator.By
import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.Until
import java.util.regex.Pattern
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_INTERACTION_TIMEOUT

fun UiDevice.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

fun UiObject2.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

fun UiDevice.findObjectWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_INTERACTION_TIMEOUT
): UiObject2 {

    wait(Until.hasObject(selector), timeout)

    return try {
        findObject(selector)
    } catch (e: NullPointerException) {
        throw IllegalArgumentException(
            "No matches for selector within timeout ($timeout): $selector"
        )
    }
}

fun UiDevice.clickAgreeOnPrivacyDisclaimer() {
    findObjectWithTimeout(By.text("Agree and continue")).click()
}

// The dialog will only be shown when there's a new version code and bundled release notes.
fun UiDevice.dismissChangelogDialogIfShown() {
    try {
        findObjectWithTimeout(By.text("Got it!")).click()
    } catch (e: IllegalArgumentException) {
        // This is OK since it means the changes dialog wasn't shown.
    }
}

fun UiDevice.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove(
    timeout: Long = DEFAULT_INTERACTION_TIMEOUT
) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
        // Skipping as notification permissions are not shown.
        return
    }

    val selector = By.text("Allow")

    wait(Until.hasObject(selector), timeout)

    try {
        findObjectWithTimeout(selector).click()
    } catch (e: IllegalArgumentException) {
        throw IllegalArgumentException(
            "Failed to allow notification permission within timeout ($timeout)"
        )
    }
}

fun UiDevice.clickOkOnVpnPermissionPrompt(timeout: Long = DEFAULT_INTERACTION_TIMEOUT) {
    try {
        findObjectWithTimeout(By.text("OK"), timeout = timeout).click()
    } catch (e: IllegalArgumentException) {
        // This is OK since it means the dialog wasn't shown and we can continue.
    }
}

fun UiObject2.findObjectWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_INTERACTION_TIMEOUT
): UiObject2 {

    wait(Until.hasObject(selector), timeout)

    return try {
        findObject(selector)
    } catch (e: NullPointerException) {
        throw IllegalArgumentException(
            "No matches for selector within timeout ($timeout): $selector"
        )
    }
}
