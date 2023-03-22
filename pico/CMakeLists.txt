cmake_minimum_required(VERSION 3.13)

# Executable name
set(NAME picosso)

# Include Pico SDK
include(lib/pico-sdk/pico_sdk_init.cmake)

# Gooey boilerplate
project(${NAME} C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Initialize the SDK
pico_sdk_init()

# Add your source files
add_executable(${NAME}
    main.cpp
)

# Include additional libraries
# include(common/pimoroni_i2c)
# include(common/pimoroni_bus)
# include(libraries/bitmap_fonts/bitmap_fonts)
# include(libraries/hershey_fonts/hershey_fonts)
# include(libraries/pico_explorer/pico_explorer)

# Link necessary libraries
target_link_libraries(${NAME}
    pico_stdlib
)

# Create map/bin/hex file etc. in addition to ELF
pico_add_extra_outputs(${NAME})

# # Set up files for the release packages
# install(FILES
#     ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.uf2
#     ${CMAKE_CURRENT_LIST_DIR}/README.md
#     DESTINATION .
# )
#
# set(CPACK_INCLUDE_TOPLEVEL_DIRECTORY OFF)
# set(CPACK_GENERATOR "ZIP" "TGZ")
# include(CPack)
