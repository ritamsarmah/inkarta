#if !defined(ARDUINO_INKPLATE10) && !defined(ARDUINO_INKPLATE10V2)
#error \
    "Wrong board selection for this example, please select e-radionica Inkplate10 or Soldered Inkplate10 in the boards menu."
#endif

#include "Inkplate.h"
#include "secrets.h"

Inkplate display(INKPLATE_1BIT);

/* Globals */

// 3.7V/4.2V battery
const double lowBatteryVoltage = 3.4;

// NOTE: width and height are switched due to portrait rotation
const int16_t widthPx = display.height();
const int16_t heightPx = display.width();

const char *host = "192.168.1.5";
const uint16_t port = 5000;

/* Utilities */

// Initializes real-time clock using server
void setRtc() {
    if (display.rtcIsSet())
        return;

    char url[256];
    sprintf(url, "http://%s:%d/device/rtc", host, port);

    HTTPClient http;
    if (http.begin(url) && http.GET() > 0) {
        int epoch = http.getString().toInt();
        display.rtcSetEpoch(epoch);

        display.println("Set RTC Epoch:" + String(epoch));
        display.display();
    }
    else {
    }
}

/* Main */

void setup() {
    display.begin();

    display.setRotation(3);
    display.setTextSize(2);
    display.setTextColor(BLACK);

    // Clear alarm flag from any previous alarm
    display.rtcClearAlarmFlag:);

    // Check for low battery
    double voltage = display.readBattery();
    if (voltage < lowBatteryVoltage) {
        display.println("Low Battery - Recharge Now");
        display.display();
        esp_deep_sleep_start();
        return;
    }

    // Connect to Wi-Fi (waits until connected)
    display.connectWiFi(ssid, password);

    setRtc();

    // Download and draw artwork
    // NOTE: Only can use Windows Bitmap file with color depth of 1, 4, 8 or 24
    // bits with no compression
    char url[256];
    sprintf(url, "http://%s:%d/image?w=%d&h=%d", host, port, widthPx, heightPx);
    if (!display.drawImage(url, display.BMP, 0, 0, false, false)) {
        display.println("Error downloading artwork");
        display.display();
        esp_deep_sleep_start();
        return;
    }

    // Refresh display to show artwork
    display.display();

    // Disconnect Wi-Fi
    display.disconnect();

    // Set wakeup at a second before midnight (11:59:59 PM)
    // Delay lets enough time pass to schedule during the next day (for the
    // correct day/weekday)
    delay(5000);
    display.rtcGetRtcData();
    display.rtcSetAlarm(59, 59, 23, display.rtcGetDay(),
                        display.rtcGetWeekday());

    esp_sleep_enable_ext0_wakeup(GPIO_NUM_36,
                                 LOW); // Enable wake via wake button
    esp_sleep_enable_ext1_wakeup(
        int64_t(1) << GPIO_NUM_39,
        ESP_EXT1_WAKEUP_ALL_LOW); // Enable wake via RTC interrupt alarm
    esp_deep_sleep_start();
}

void loop() {
    // Never here, since deep sleep restarts board every time
}
