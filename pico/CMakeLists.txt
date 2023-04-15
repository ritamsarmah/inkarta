cmake_minimum_required(VERSION 3.13)

set(NAME picosso)
set(ENV{PICO_SDK_PATH} "../../pico-sdk/")
set(CMAKE_BUILD_TYPE Release)

include(pico_sdk_import.cmake)

project(${NAME} C CXX ASM)
set(PICO_BOARD "pico_w")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

pico_sdk_init()

add_executable(${NAME}
    main.cpp
)

target_compile_options(${NAME} PRIVATE -Os)

target_include_directories(${NAME} PRIVATE ${CMAKE_CURRENT_LIST_DIR})

target_link_options(${NAME} PRIVATE -s)

target_link_libraries(${NAME}
    pico_stdlib
    pico_cyw43_arch_lwip_threadsafe_background
    hardware_flash
)

# Enable usb output, disable uart output
pico_enable_stdio_usb(${NAME} 1)
pico_enable_stdio_uart(${NAME} 0)

# Create map/bin/hex file etc. in addition to ELF
pico_add_extra_outputs(${NAME})
