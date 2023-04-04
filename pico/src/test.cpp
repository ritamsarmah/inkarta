#include <cstdio>
#include <string>

// Pico W Specs: 264 KB SRAM, 16 kB on-chip cache, 2MB of Flash Memory

using namespace std;

/* Globals */

const uint16_t width_px = 296;
const uint16_t height_px = 128;

const string host = "127.0.0.1";
const string path =
    "/image?w=" + to_string(width_px) + "&h=" + to_string(height_px);
const uint32_t port = 5000;
const string request = "GET " + path + " HTTP/1.1\r\n\
    Host: " + host + "\r\n\
    Connection: close\r\n\r\n";

const uint32_t max_file_size = 1 * 1024 * 1024; // 1 MB
const uint32_t buffer_size = 8192;              // 8 KB
uint8_t response_buffer[buffer_size];

// Fetch a random image to fit display
void fetch_image() {

    // Open the remote file: Create a socket and connect to the server. Send an
    // HTTP GET request for the remote file to the server, including any
    // necessary headers.
    //
    // Create a local file: Open a local file for writing, and allocate a buffer
    // of a fixed size.
    //
    // Download the file: Use the recv() function to read data from the socket
    // in chunks, and write each chunk to the local file. Repeat this process
    // until the entire file has been downloaded.
    //
    // Ultimately need to ensure constant memory usage on Pico's limited memory
    // regardless of file size (which can vary depending on the requested
    // width/height dimensions)

    // int sock = socket(AF_INET, SOCK_STREAM, 0);
    //
    // struct sockaddr_in server;
    // server.sin_addr.s_addr = inet_addr("127.0.0.1");
    // server.sin_family = AF_INET;
    // server.sin_port = htons(80);
    //
    // connect(sock, (struct sockaddr *)&server, sizeof(server));
    //
    // std::string request = "GET /large_file.bin HTTP/1.1\r\nHost:
    // localhost\r\n\r\n"; send(sock, request.c_str(), request.length(), 0);
    //
    // std::ofstream file("large_file.bin", std::ios::binary);
    // const int buffer_size = 4096;
    // char buffer[buffer_size];
    // int bytes_received;
    //
    // do {
    //   bytes_received = recv(sock, buffer, buffer_size, 0);
    //   if (bytes_received > 0) {
    //     file.write(buffer, bytes_received);
    //   }
    // } while (bytes_received > 0);
    //
    // file.close();
    // close(sock);

    printf("%s", request.c_str());

    // TODO: Make HTTP request to server

    // 1. stream data into bitset
    //  - draw to display by parsing through bitset smartly
    // 2. stream data into image file
    //  - easier to display probably with direct image
    // 3. Use mmap to map file to memory

    // Make sure to profile the memory usage
}

// Print image to display
void print_image() {}

int main() {
    printf("Hello world!\n");
    // fetch_image();
    return 0;
}
