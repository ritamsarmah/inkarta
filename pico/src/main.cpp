// TODO: Use lwip/apps/http_client.h???
#include "pico/cyw43_arch.h"
#include "pico/stdlib.h"
#include "secrets.hpp"

#include <cstdio>
#include <string>

using namespace std;

// TODO: test is to connect to /next and print the output to stdout

int main() {
    stdio_init_all(); // TODO: Remove after debugging finished

    // https://datasheets.raspberrypi.com/picow/connecting-to-the-internet-with-pico-w.pdf

    if (cyw43_arch_init()) {
        printf("Failed to initialize\n");
        return 1;
    }

    printf("Connecting to Wi-Fi: %s\n", secret::wifi_ssid);
    if (cyw43_arch_wifi_connect_timeout_ms(secret::wifi_ssid,
                                           secret::wifi_password,
                                           CYW43_AUTH_WPA2_AES_PSK, 30000)) {
        printf("Failed to connect to Wi-Fi\n");
        return 1;
    }

    printf("Connected to Wi-Fi\n");
    return 0;
}
