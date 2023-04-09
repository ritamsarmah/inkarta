#include "hardware/flash.h"
#include "lwip/tcp.h"
#include "pico/cyw43_arch.h"
#include "pico/stdlib.h"
#include "secret.hpp"

#include <cstdio>
#include <string>

// Pico W Specs: 264 KB SRAM, 16 kB on-chip cache, 2MB of Flash Memory

/* Forward Declarations */

err_t tcp_client_connected(void *arg, struct tcp_pcb *tpcb, err_t err);
err_t tcp_client_recv(void *arg, struct tcp_pcb *tpcb, struct pbuf *p,
                      err_t err);
void tcp_client_err(void *arg, err_t err);

/* Globals */

const u32_t width_px = 296;
const u32_t height_px = 128;

const u32_t country = CYW43_COUNTRY_USA;
const u32_t auth = CYW43_AUTH_WPA2_AES_PSK;

const std::string host = "192.168.1.25"; // TODO: Change to rpi (5)
const u16_t port = 5000;

const std::string path = "/next";
// const std::string path =
//     "/image?w=" + std::to_string(width_px) + "&h=" +
//     std::to_string(height_px);
const std::string request = "GET " + path + " HTTP/1.1\r\n\
    Host: " + host + "\r\n\
    Connection: close\r\n\r\n";

const u8_t poll_interval_s = 10;
const u32_t buffer_size = 8 * 1024; // 8 KB

// TCP connection state
typedef struct tcp_client_state {
    struct tcp_pcb *tpcb;
    u8_t buffer[buffer_size];
    int buffer_len;
    int recv_count;
    bool connected;
    bool completed;
} tcp_client_state;

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

void wifi_disconnect() {
    printf("Disconnected from Wi-Fi\n");
    cyw43_arch_deinit();
}

/* TCP */

tcp_client_state *tcp_client_init() {
    tcp_client_state *state = new tcp_client_state;

    state->tpcb = tcp_new();
    if (state->tpcb == NULL) {
        printf("Failed to create TCP PCB\n");
        return NULL;
    }

    state->buffer_len = 0;
    state->recv_count = 0;
    state->connected = false;
    state->completed = false;

    return state;
}

bool tcp_client_open(tcp_client_state *state) {
    tcp_arg(state->tpcb, state);
    tcp_recv(state->tpcb, tcp_client_recv);
    tcp_err(state->tpcb, tcp_client_err);

    ip_addr_t server_addr;
    ip4_addr_set_u32(&server_addr, ipaddr_addr(host.c_str()));

    cyw43_arch_lwip_begin();
    err_t err =
        tcp_connect(state->tpcb, &server_addr, port, tcp_client_connected);
    cyw43_arch_lwip_end();

    return err == ERR_OK;
}

err_t tcp_client_close(tcp_client_state *state) {
    if (state->tpcb == NULL) return ERR_OK;

    tcp_arg(state->tpcb, NULL);
    tcp_recv(state->tpcb, NULL);
    tcp_err(state->tpcb, NULL);

    err_t err = tcp_close(state->tpcb);
    if (err != ERR_OK) {
        printf("Failed to close TCP connection %d. Aborting...\n", err);
        tcp_abort(state->tpcb);
        err = ERR_ABRT;
    }

    return err;
}

err_t tcp_client_finish(tcp_client_state *state, int status,
                        std::string message) {
    printf("%s (%d)\n", message.c_str(), status);

    state->completed = true;
    err_t err = tcp_client_close(state);
    state->connected = false;

    return err;
}

/* TCP Callbacks */

err_t tcp_client_connected(void *arg, struct tcp_pcb *tpcb, err_t err) {
    tcp_client_state *state = (tcp_client_state *)arg;

    if (err != ERR_OK) {
        return tcp_client_finish(state, err, "Failed to connect to server");
    }

    printf("Connected to server: %s\n", host.c_str());
    state->connected = true;
    return ERR_OK;
}

err_t tcp_client_recv(void *arg, struct tcp_pcb *tpcb, struct pbuf *p,
                      err_t err) {
    tcp_client_state *state = (tcp_client_state *)arg;

    // A NULL pbuf indicates remote host has closed the connection
    if (p == NULL) {
        return tcp_client_finish(state, 0, "Connection closed");
    }

    state->recv_count++;

    cyw43_arch_lwip_check();
    if (p->tot_len > 0) {
        // Receive the buffer
        const u16_t buffer_left = buffer_size - state->buffer_len;
        state->buffer_len += pbuf_copy_partial(
            p, state->buffer + state->buffer_len,
            p->tot_len > buffer_left ? buffer_left : p->tot_len, 0);
        tcp_recved(tpcb, p->tot_len);

        printf("%.*s\n", state->buffer_len, state->buffer);
    }

    pbuf_free(p);

    return ERR_OK;
}

void tcp_client_err(void *arg, err_t err) {
    if (err != ERR_ABRT) {
        tcp_client_finish((tcp_client_state *)arg, err,
                          "An error occurred with the TCP connection");
    }
}

/* High-Level Logic */

void print_image() {
    // TODO: Print image to e-ink display
}

bool download_image() {
    tcp_client_state *state = tcp_client_init();

    // Open connection to server
    if (state == NULL || !tcp_client_open(state)) {
        tcp_client_finish(state, -1, "Failed to open TCP connection");
        delete state;
        return false;
    }

    // Wait until connected to server
    while (!state->connected) {
        cyw43_arch_wait_for_work_until(make_timeout_time_ms(1000));
    }

    // Send HTTP request
    err_t err = tcp_write(state->tpcb, request.c_str(), request.length(),
                          TCP_WRITE_FLAG_COPY);
    if (err != ERR_OK) {
        tcp_client_finish(state, -1, "Failed to send HTTP request");
        delete state;
        return false;
    }

    // Wait for response
    while (!state->completed) {
        cyw43_arch_poll();
        cyw43_arch_wait_for_work_until(make_timeout_time_ms(1000));
    }

    delete state;
    return true;
}

void update_image() {
    if (!wifi_connect()) return;

    if (!download_image()) {
        printf("Failed to download image\n");
        return;
    }

    // print_image();

    // if (schedule) {
    // TODO: Schedule next update?
    // }

    wifi_disconnect();
}

/* Display */

void schedule_update_image() {
    // TODO: get current time and schedule at midnight
}

int main() {
    stdio_init_all();

    sleep_ms(2000); // TODO: remove after debug

    // TODO: Button should force trigger new fetch
    // TODO: check if the connected = true, if it's true then update already
    // happening so don't redo!

    update_image();

    return 0;
}
