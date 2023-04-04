// TODO: Use lwip/apps/http_client.h???
#include <cstdio>
#include <string>

#include "pico/cyw43_arch.h"
#include "pico/stdlib.h"
#include "secrets.hpp"

using namespace std;

// TODO: test is to connect to /next and print the output to stdout

int main() {
    stdio_init_all();

    // https://datasheets.raspberrypi.com/picow/connecting-to-the-internet-with-pico-w.pdf
    if (cyw43_arch_init_with_country(CYW43_COUNTRY_USA)) {
        printf("Failed to initialize\n");
        return 1;
    }

    cyw43_arch_enable_sta_mode();

    printf("Connecting to Wi-Fi: %s\n", secret::wifi_ssid.c_str());
    if (cyw43_arch_wifi_connect_timeout_ms(secret::wifi_ssid.c_str(),
                                           secret::wifi_password.c_str(),
                                           CYW43_AUTH_WPA2_AES_PSK, 10000)) {
        printf("Failed to connect to Wi-Fi\n");
        return 1;
    }

    printf("Connected to Wi-Fi\n");
    return 0;
}
