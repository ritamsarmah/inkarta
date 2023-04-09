#include "hardware/flash.h"
#include "hardware/sync.h"
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
const u8_t poll_interval_s = 10;

const std::string host = "192.168.1.25"; // TODO: Change to rpi (5)
const u16_t port = 5000;

const std::string path =
    "/image?w=" + std::to_string(width_px) + "&h=" + std::to_string(height_px);
const std::string request = "GET " + path + " HTTP/1.1\r\n\
    Host: " + host + "\r\n\
    Connection: close\r\n\r\n";

// Data
const u32_t buffer_size = FLASH_PAGE_SIZE * 16; // 4 KB

/**
 * Use region of 64KB from end of flash memory for storing image.
 * This is to avoid overwriting code written at the front of flash.
 *
 * NOTE: Whole number of sectors must be erased at a time, hence the
 * target size being specified with FLASH_SECTOR_SIZE for ease of use.
 */
const size_t flash_target_size = FLASH_SECTOR_SIZE * 16; // 64 KB
const u32_t flash_target_offset = PICO_FLASH_SIZE_BYTES - flash_target_size;
const u8_t *flash_target_data = (const u8_t *)(XIP_BASE + flash_target_offset);

// TCP connection state
typedef struct tcp_client_state {
    struct tcp_pcb *tpcb;
    u8_t buffer[buffer_size];
    size_t buffer_len;
    size_t flash_len;
    u32_t recv_count;
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
    state->flash_len = 0;
    state->recv_count = 0;
    state->connected = false;
    state->completed = false;

    // Prepare flash target region for storing image data
    uint32_t ints = save_and_disable_interrupts();
    flash_range_erase(flash_target_offset, flash_target_size);
    restore_interrupts(ints);

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

/* Flash */

void print_buffer(const u8_t *buf, size_t len) {
    printf("\n--- Start Buffer ---\n");
    for (size_t i = 0; i < len; ++i) {
        printf("%02x", buf[i]);
        if (i % 16 == 15)
            printf("\n");
        else
            printf(" ");
    }
    printf("\n--- End Buffer ---\n");
}

err_t flash_write(tcp_client_state *state, bool flush) {
    if (state->buffer_len == 0) return ERR_OK;

    // Calculate number of extra bytes that don't align with page size
    const size_t remainder = state->buffer_len % FLASH_PAGE_SIZE;
    size_t flash_write_len;

    if (flush) {
        // Write buffer contents WITH padding to align with page size
        const size_t padding = FLASH_PAGE_SIZE - remainder;
        flash_write_len = state->buffer_len + padding;

        printf("[Flush] Writing %d bytes to flash memory (padded %d bytes) \n",
               state->buffer_len, padding);

    } else {
        // Only write as much data that aligns with page size
        flash_write_len = state->buffer_len - remainder;

        printf("[Partial] Writing %d bytes to flash memory\n", flash_write_len);
    }

    // Check if not enough data to fit page boundary
    if (flash_write_len < FLASH_PAGE_SIZE) return ERR_OK;

    // Check if writing additional data will exceed target size
    if (state->flash_len + flash_write_len >= flash_target_size) {
        return tcp_client_finish(
            state, ERR_MEM, "Response data is larger than target flash size");
    }

    u32_t flash_offset = flash_target_offset + state->flash_len;

    // Program flash
    uint32_t ints = save_and_disable_interrupts();
    flash_range_program(flash_offset, state->buffer, flash_write_len);
    restore_interrupts(ints);

    // If data is left over in buffer, move it to beginning
    if (!flush && remainder != 0) {
        memmove(state->buffer, state->buffer + flash_write_len, remainder);
    }

    // NOTE: After flushing the buffer, the flash_len reflects the true length
    // of data, i.e., not including any padding to align to page size
    const size_t delta_len = flush ? state->buffer_len : flash_write_len;
    state->flash_len += delta_len;
    state->buffer_len -= delta_len;

    return ERR_OK;
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
        flash_write(state, true);
        return tcp_client_finish(state, 0, "Connection closed");
    }

    state->recv_count++;

    cyw43_arch_lwip_check();
    if (p->tot_len > 0) {
        // Receive the buffer if it contains the response data (ignores headers)
        if (state->recv_count > 1) {
            const u16_t buffer_left = buffer_size - state->buffer_len;
            state->buffer_len += pbuf_copy_partial(
                p, state->buffer + state->buffer_len,
                p->tot_len > buffer_left ? buffer_left : p->tot_len, 0);
        }

        tcp_recved(tpcb, p->tot_len);
        ("%.*s\n", state->buffer_len, state->buffer);
    }

    pbuf_free(p);

    // Write buffer contents to flash if needed
    return flash_write(state, false);
}

void tcp_client_err(void *arg, err_t err) {
    if (err != ERR_ABRT) {
        tcp_client_finish((tcp_client_state *)arg, err,
                          "An error occurred with the TCP connection");
    }
}

/* High-Level Logic */

void print_image(size_t data_len) {
    printf("Printing image (%d bytes)...\n", data_len);
    print_buffer(flash_target_data, data_len);
}

// Returns the size of the image (stored in flash memory)
size_t download_image() {
    tcp_client_state *state = tcp_client_init();

    // Open connection to server
    if (state == NULL || !tcp_client_open(state)) {
        tcp_client_finish(state, -1, "Failed to open TCP connection");
        delete state;
        return -1;
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
        return -1;
    }

    // Wait for response
    while (!state->completed) {
        cyw43_arch_poll();
        cyw43_arch_wait_for_work_until(make_timeout_time_ms(1000));
    }

    size_t data_len = state->flash_len;
    delete state;

    return data_len;
}

void update_image() {
    if (!wifi_connect()) return;

    size_t data_len = download_image();
    if (data_len <= 0) {
        printf("Failed to download image\n");
        return;
    }

    print_image(data_len);

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
