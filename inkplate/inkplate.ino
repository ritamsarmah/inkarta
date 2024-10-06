#if !defined(ARDUINO_INKPLATE10) && !defined(ARDUINO_INKPLATE10V2)
#error                                                                         \
    "Wrong board selection for this example, please select e-radionica Inkplate10 or Soldered Inkplate10 in the boards menu."
#endif

#include "Inkplate.h"
#include "secrets.h"

Inkplate display(INKPLATE_3BIT);

/* Globals */

constexpr double lowBatteryVoltage = 3.4; // For 3.7V/4.2V battery
char url[128];                            // URL Buffer

/* Utilities */

// Fetch epoch value from server for specified endpoint
bool fetchEpoch(const char *endpoint, void (*set)(uint32_t)) {
    snprintf(url, sizeof(url), "http://%s:%d/device/%s", host, port, endpoint);

    HTTPClient http;
    bool success = false;
    if (http.begin(url) && http.GET() == HTTP_CODE_OK) {
        uint32_t epoch = http.getString().toInt();
        if (epoch > 0) {
            set(epoch);
            success = true;
        }
    }

    http.end();
    return success;
}

// Initializes real-time clock using server
bool setRtc() {
    return fetchEpoch("rtc",
                      [](uint32_t epoch) { display.rtcSetEpoch(epoch); });
}

// Set alarm for next display refresh using server
bool setAlarm() {
    return fetchEpoch("alarm", [](uint32_t epoch) {
        display.rtcSetAlarmEpoch(epoch, RTC_ALARM_MATCH_DHHMMSS);
    });
}

// Print error message and sleep display
void displayError(const char *message) {
    display.println(message);
    display.display();
    display.disconnect();
    esp_deep_sleep_start();
}

/* Main */

void setup() {
    display.begin();

    display.setRotation(3);
    display.setTextSize(2);
    display.setTextColor(BLACK);

    // Reset real-time clock
    // display.rtcReset();

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
    if (!display.rtcIsSet() && !setRtc()) {
        displayError("Failed to set real time clock");
        return;
    }

    // Download and draw artwork
    snprintf(url, sizeof(url), "http://%s:%d/image/next?width=%d&height=%d",
             host, port, display.width(), display.height());
    if (!display.drawImage(url, display.PNG, 0, 0, false, false)) {
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

    // Explicitly urn off power supply for SD card (even though it's not used)
    display.sdCardSleep();

    // FIXME: Enable wake via wake button
    esp_sleep_enable_ext0_wakeup(GPIO_NUM_36, LOW);

    // Enable wake via RTC interrupt alarm
    esp_sleep_enable_ext1_wakeup(int64_t(1) << GPIO_NUM_39,
                                 ESP_EXT1_WAKEUP_ALL_LOW);

    // Enter ESP32 low power mode
    esp_deep_sleep_start();
}

void loop() { /* Never here, since deep sleep restarts board every time */ }
