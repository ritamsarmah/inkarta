#if !defined(ARDUINO_INKPLATE10) && !defined(ARDUINO_INKPLATE10V2)
#error "Wrong board selection for this example, please select e-radionica Inkplate10 or Soldered Inkplate10 in the boards menu."
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

String getValueForKey(String data, String key) {
  int keyIndex = data.indexOf(key);
  if (keyIndex == -1) return "";

  int valueIndex = data.indexOf(": ", keyIndex) + 2;
  int endIndex = data.indexOf('\n', valueIndex);
  
  return data.substring(valueIndex, endIndex);
}

// Sets RTC to current datetime based on WorldTimeAPI
void rtcSetDateTime() {
  if (display.rtcIsSet()) return;

  HTTPClient http;
  if (http.begin("http://worldtimeapi.org/api/ip.txt") && http.GET() > 0) {
    String response = http.getString();
    int epoch = getValueForKey(response, "unixtime").toInt();
    display.rtcSetEpoch(epoch);
    
    display.println("Set RTC Epoch:" + String(epoch));
    display.display();
  }
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
  if ( voltage < lowBatteryVoltage ) {
    display.println("Low Battery - Recharge Now");
    display.display();
    esp_deep_sleep_start();
    return;
  }
  
  // Connect to Wi-Fi (waits until connected)
  display.connectWiFi(ssid, password);

  // Set RTC if needed
  rtcSetDateTime();

  // Download and draw artwork
  // NOTE: Only can use Windows Bitmap file with color depth of 1, 4, 8 or 24 bits with no compression
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
  // Delay lets enough time pass to schedule during the next day (for the correct day/weekday)
  delay(5000);
  display.rtcGetRtcData();
  display.rtcSetAlarm(59, 59, 23, display.rtcGetDay(), display.rtcGetWeekday());

  esp_sleep_enable_ext0_wakeup(GPIO_NUM_36, LOW); // Enable wake via wake button
  esp_sleep_enable_ext1_wakeup(int64_t(1) << GPIO_NUM_39, ESP_EXT1_WAKEUP_ALL_LOW); // Enable wake via RTC interrupt alarm
  esp_deep_sleep_start();
}

void loop() {
  // Never here, since deep sleep restarts board every time
}
