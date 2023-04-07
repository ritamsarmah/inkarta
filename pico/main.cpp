#include "lwip/tcp.h"
#include "pico/cyw43_arch.h"
#include "pico/stdlib.h"
#include "secret.hpp"

#include <algorithm>
#include <cstdio>
#include <string>

// Pico W Specs: 264 KB SRAM, 16 kB on-chip cache, 2MB of Flash Memory

/* Globals */

const uint16_t width_px = 296;
const uint16_t height_px = 128;

const uint32_t country = CYW43_COUNTRY_USA;
const uint32_t auth = CYW43_AUTH_WPA2_AES_PSK;

const std::string host = "192.168.1.25"; // TODO: Change to rpi (5)
const uint16_t port = 5000;

const std::string path =
    "/image?w=" + std::to_string(width_px) + "&h=" + std::to_string(height_px);
const std::string request = "GET " + path + " HTTP/1.1\r\n\
    Host: " + host + "\r\n\
    Connection: close\r\n\r\n";

const uint8_t poll_interval_s = 10;
const uint32_t max_file_size = 1 * 1024 * 1024; // 1 MB
const uint32_t buffer_size = 8 * 1024;          // 8 KB

typedef struct tcp_client_state {
    struct tcp_pcb *tcp_pcb;
    uint8_t buffer[buffer_size];
    int buffer_len;
    bool completed;
    bool connected;
} tcp_client_state;

tcp_client_state state;

err_t tcp_client_connected(void *arg, struct tcp_pcb *tpcb, err_t err);
err_t tcp_client_recv(void *arg, struct tcp_pcb *tpcb, struct pbuf *p,
                      err_t err);
void tcp_client_err(void *arg, err_t err);

/* Networking */

bool wifi_connect() {
    if (cyw43_arch_init_with_country(country)) {
        printf("Failed to initialize\n");
        return false;
    }

    cyw43_arch_enable_sta_mode();

    printf("Connecting to Wi-Fi (%s)...\n", secret::ssid);
    if (cyw43_arch_wifi_connect_timeout_ms(secret::ssid, secret::password, auth,
                                           10000)) {
        printf("Failed to connect to Wi-Fi\n");
        return false;
    }

    printf("Connected to Wi-Fi\n");
    return true;
}

bool wifi_disconnect() {
    printf("Disconnected from Wi-Fi\n");
    cyw43_arch_deinit();
}

/* TCP */

void tcp_client_init(tcp_client_state *state) {
    state->tcp_pcb = tcp_new();
    std::fill(state->buffer, buffer_size, 0);
    state->buffer_len = 0;
    state->completed = false;
    state->connected = false;
}

err_t tcp_client_close(tcp_client_state *state) {
    if (state->tcp_pcb == NULL) return ERR_OK;

    tcp_arg(state->tcp_pcb, NULL);
    tcp_recv(state->tcp_pcb, NULL);
    tcp_err(state->tcp_pcb, NULL);

    err_t err = tcp_close(state->tcp_pcb);
    if (err != ERR_OK) {
        printf("Failed to close TCP connection %d. Aborting...\n", err);
        tcp_abort(state->tcp_pcb);
        err = ERR_ABRT;
    }

    return err;
}

err_t tcp_client_open(tcp_client_state *state) {
    tcp_arg(state->tcp_pcb, state);
    tcp_recv(state->tcp_pcb, tcp_client_recv);
    tcp_err(state->tcp_pcb, tcp_client_err);

    state->buffer_len = 0;

    ip_addr_t server_addr;
    ip4_addr_set_u32(&server_addr, ipaddr_addr(host.c_str()));

    return tcp_connect(state->tcp_pcb, &server_addr, port,
                       tcp_client_connected);
}

err_t tcp_finish(tcp_client_state *state, int status, std::string msg) {
    printf("%s (%d)\n", msg.c_str(), status);
    state->completed = true;
    return tcp_client_close(state);
}

/* TCP Callbacks */

err_t tcp_client_connected(void *arg, struct tcp_pcb *tpcb, err_t err) {
    tcp_client_state *state = (tcp_client_state *)arg;
    if (err != ERR_OK) {
        return tcp_finish(state, err, "Failed to connect to server");
    }

    printf("Connected to server: %s\n", host.c_str());
    state->connected = true;
    return ERR_OK;
}

err_t tcp_client_recv(void *arg, struct tcp_pcb *tpcb, struct pbuf *p,
                      err_t err) {
    tcp_client_state *state = (tcp_client_state *)arg;

    // A NULL pbuf indicates remote host has closed the connection
    if (!p) {
        return tcp_finish(state, 0, "Connection closed");
    }

    cyw43_arch_lwip_check();
    if (p->tot_len > 0) {
        printf("recv %d err %d\n", p->tot_len, err); // TODO: Remove debug

        // For 296x128 image -> file size is 5KB
        // For 800x480 image -> file size is 48KB

        // Receive the buffer
        // TODO: what happens if the buffer size is smaller than the provided p?
        const uint16_t buffer_left = buffer_size - state->buffer_len;
        state->buffer_len += pbuf_copy_partial(
            p, state->buffer + state->buffer_len,
            p->tot_len > buffer_left ? buffer_left : p->tot_len, 0);
        tcp_recved(tpcb, p->tot_len);

        printf("%s\n", state->buffer);
        // TODO: write to file
        // Extract only data from response

        // Clear buffer
        std::fill(state->buffer, state->buffer + state->buffer_len, 0);
        state->buffer_len = 0;
    }

    pbuf_free(p);

    return ERR_OK;
}

void tcp_client_err(void *arg, err_t err) {
    if (err != ERR_ABRT) {
        tcp_finish((tcp_client_state *)arg, err,
                   "An error occurred with the TCP connection");
    }
}

/* High-Level Logic */

void print_image(uint8_t buffer, int length) {
    // TODO: Print image to e-ink display
}

bool update_image() {
    // Initialize Wi-Fi
    if (!wifi_connect()) return false;

    // Initialize TCP client state


    // Open connection to server
    err_t err = tcp_client_open(state);
    if (err != ERR_OK) {
        tcp_finish(state, -1, "Failed to open TCP connection");
        return false;
    }

    // Wait until connected to server
    while (!state->connected) {
        cyw43_arch_wait_for_work_until(make_timeout_time_ms(1000));
    }

    // Send HTTP request
    err = tcp_write(state->tcp_pcb, request.c_str(), request.length(),
                    TCP_WRITE_FLAG_COPY);
    if (err != ERR_OK) {
        return tcp_finish(state, -1, "Failed to send HTTP request");
    }

    // Wait for response
    while (!state->completed) {
        cyw43_arch_poll();
        cyw43_arch_wait_for_work_until(make_timeout_time_ms(1000));
    }

    // Update display
    print_image(state->buffer, state->buffer_len);

    wifi_disconnect();
    return tcp_finish(state, 0, "Successfully updated image");
}

/* Display */

void schedule_update_image() {
    // TODO: get current time
}

int main() {
    stdio_init_all();

    sleep_ms(2000); // TODO: remove after debug

    while (true) {
        // TODO: Check the current time

        // TODO
        // Infinite loop -> schedule at midnight to fetch new image (async?)
        // Button should force trigger new fetch
        // Disconnect Wifi while waiting!

        fetch_image(); // TODO: Check if failed, and sleep and try again
    }

    return 0;
}
