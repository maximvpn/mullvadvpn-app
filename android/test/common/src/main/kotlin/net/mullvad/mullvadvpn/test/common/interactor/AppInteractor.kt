package net.mullvad.mullvadvpn.test.common.interactor

import android.content.Context
import android.content.Intent
import android.widget.Button
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.endpoint.CustomApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.test.common.constant.AFTER_SCROLL_INTERACTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.APP_LAUNCH_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.CONNECTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_INTERACTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_FAILURE_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_PROMPT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context,
    private val targetPackageName: String
) {
    fun launch(customApiEndpointConfiguration: CustomApiEndpointConfiguration? = null) {
        device.pressHome()
        // Wait for launcher
        device.wait(
            Until.hasObject(By.pkg(device.launcherPackageName).depth(0)),
            APP_LAUNCH_TIMEOUT
        )

        val intent =
            targetContext.packageManager.getLaunchIntentForPackage(targetPackageName)?.apply {
                // Clear out any previous instances
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
                if (customApiEndpointConfiguration != null) {
                    putApiEndpointConfigurationExtra(customApiEndpointConfiguration)
                }
            }
        targetContext.startActivity(intent)
        device.wait(Until.hasObject(By.pkg(targetPackageName).depth(0)), APP_LAUNCH_TIMEOUT)
    }

    fun launchAndEnsureLoggedIn(accountToken: String) {
        launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        waitForLoginPrompt()
        attemptLogin(accountToken)
        ensureLoggedIn()
    }

    fun launchAndCreateAccount() {
        launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        waitForLoginPrompt()
        attemptCreateAccount()
        ensureAccountCreated()
    }

    fun attemptLogin(accountToken: String) {
        val loginObject =
            device.findObjectWithTimeout(By.clazz("android.widget.EditText")).apply {
                text = accountToken
            }
        loginObject.parent.findObject(By.clazz(Button::class.java)).click()
    }

    fun attemptCreateAccount() {
        device.findObjectWithTimeout(By.text("Create account")).click()
    }

    fun ensureAccountCreated(accountToken: String? = null) {
        device.findObjectWithTimeout(By.text("Congrats!"), LOGIN_TIMEOUT)
        accountToken?.let {
            device.findObjectWithTimeout(By.text(accountToken), DEFAULT_INTERACTION_TIMEOUT)
        }
    }

    fun ensureAccountCreationFailed() {
        device.findObjectWithTimeout(By.text("Failed to create account"), LOGIN_FAILURE_TIMEOUT)
    }

    fun ensureLoggedIn() {
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"), LOGIN_TIMEOUT)
    }

    fun ensureOutOfTime() {
        device.findObjectWithTimeout(By.res("out_of_time_screen_title_test_tag"))
    }

    fun ensureAccountScreen() {
        device.findObjectWithTimeout(By.text("Account"))
    }

    fun extractIpAddress(): String {
        device.findObjectWithTimeout(By.res("location_info_test_tag")).click()
        return device
            .findObjectWithTimeout(
                By.res("location_info_connection_out_test_tag"),
                CONNECTION_TIMEOUT
            )
            .text
            .extractIpAddress()
    }

    fun clickSettingsCog() {
        device.findObjectWithTimeout(By.res("top_bar_settings_button")).click()
    }

    fun clickAccountCog() {
        device.findObjectWithTimeout(By.res("top_bar_account_button")).click()
    }

    fun clickListItemByText(text: String, timeout: Long = DEFAULT_INTERACTION_TIMEOUT) {
        device.findObjectWithTimeout(By.text(text), timeout).click()
    }

    fun clickListItemByRes(res: String, timeout: Long = DEFAULT_INTERACTION_TIMEOUT) {
        device.findObjectWithTimeout(By.res(res), timeout).click()
    }

    fun clickActionButtonByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun waitForLoginPrompt(timeout: Long = LOGIN_PROMPT_TIMEOUT) {
        device.findObjectWithTimeout(By.text("Login"), timeout)
    }

    fun attemptToRemoveDevice() {
        device.findObjectWithTimeout(By.desc("Remove")).click()
        clickActionButtonByText("Yes, log out device")
    }

    fun scrollToListItemAndClick(listTag: String, itemRes: String) {
        device
            .findObjectWithTimeout(By.res(listTag))
            .scrollUntil(Direction.DOWN, Until.findObject(By.res(itemRes)))
        clickListItemByRes(itemRes, AFTER_SCROLL_INTERACTION_TIMEOUT)
    }

    private fun String.extractIpAddress(): String {
        return split(" ")[1].split(" ")[0]
    }
}
