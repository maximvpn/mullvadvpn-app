package net.mullvad.mullvadvpn.widget

import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver

class MullvadWidgetReceiver(
    override val glanceAppWidget: GlanceAppWidget = MullvadAppWidget()
) : GlanceAppWidgetReceiver() {

}
