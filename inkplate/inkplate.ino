#if !defined(ARDUINO_INKPLATE10) && !defined(ARDUINO_INKPLATE10V2)
#error                                                                         \
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

const char *host = "192.168.1.25";
const uint16_t port = 5000;

/* Utilities */

// Initializes real-time clock using server
bool setRtc() {
    if (display.rtcIsSet()) return true;

    char url[256];
    sprintf(url, "http://%s:%d/device/rtc", host, port);

    HTTPClient http;
    if (http.begin(url) && http.GET() == HTTP_CODE_OK) {
        int epoch = http.getString().toInt();
        display.rtcSetEpoch(epoch);
        return true;
    }

    return false;
}

// Set alarm for next display refresh using server
bool setAlarm() {
    char url[256];
    sprintf(url, "http://%s:%d/device/alarm", host, port);

    HTTPClient http;
    if (http.begin(url) && http.GET() == HTTP_CODE_OK) {
        int epoch = http.getString().toInt();
        display.rtcSetAlarmEpoch(epoch, RTC_ALARM_MATCH_DHHMMSS);
        return true;
    }

    return false;
}

// Print error message and sleep display
void displayError(const char *message) {
    display.println(message);
    display.display();
    esp_deep_sleep_start();
}

/* Main */

void setup() {
    display.begin();

    display.setRotation(3);
    display.setTextSize(2);
    display.setTextColor(BLACK);

    // Clear alarm flag from any previous alarm
    display.rtcClearAlarmFlag();

    // Check for low battery
    double voltage = display.readBattery();
    if (voltage < lowBatteryVoltage) {
        displayError("Low Battery - Recharge Now");
        return;
    }

    // Connect to Wi-Fi (waits until connected)
    if (!display.connectWiFi(ssid, password)) {
        displayError("Failed to connect to Wi-Fi");
        return;
    }

    // Set real-time clock if needed
    if (!setRtc()) {
        displayError("Failed to set real time clock");
        return;
    }

    // Download and draw artwork
    char url[256];
    sprintf(url, "http://%s:%d/image/next?width=%d&height=%d", host, port,
            widthPx, heightPx);
    if (!display.drawImage(url, display.BMP, 0, 0, false, false)) {
        displayError("Error downloading artwork");
        return;
    }

    // Refresh display to show artwork
    display.display();

    // Set next display refresh alarm
    if (!setAlarm()) {
        displayError("Failed to set alarm");
        return;
    }

    // Disconnect Wi-Fi
    display.disconnect();

    esp_sleep_enable_ext0_wakeup(GPIO_NUM_36, LOW); // Enable wake via wake button
    esp_sleep_enable_ext1_wakeup(
        int64_t(1) << GPIO_NUM_39,
        ESP_EXT1_WAKEUP_ALL_LOW); // Enable wake via RTC interrupt alarm
    esp_deep_sleep_start();
}

void loop() {
    // Never here, since deep sleep restarts board every time
}
