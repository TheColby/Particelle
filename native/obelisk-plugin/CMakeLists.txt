cmake_minimum_required(VERSION 3.22)
project(ObeliskM0Plugin VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(JUCE_SOURCE_DIR "" CACHE PATH "Local JUCE source checkout")
if(JUCE_SOURCE_DIR)
  if(NOT EXISTS "${JUCE_SOURCE_DIR}/CMakeLists.txt")
    message(FATAL_ERROR "JUCE_SOURCE_DIR is not a JUCE checkout")
  endif()
  add_subdirectory("${JUCE_SOURCE_DIR}" "${CMAKE_BINARY_DIR}/JUCE")
else()
  find_package(JUCE CONFIG REQUIRED)
endif()

set(OBELISK_RUST_LIB "" CACHE FILEPATH "Built obelisk_m0 Rust static library")
if(NOT EXISTS "${OBELISK_RUST_LIB}")
  message(FATAL_ERROR "Build the Rust staticlib first and set OBELISK_RUST_LIB")
endif()
add_library(obelisk_m0_rust STATIC IMPORTED GLOBAL)
set_target_properties(obelisk_m0_rust PROPERTIES IMPORTED_LOCATION "${OBELISK_RUST_LIB}")

set(OBELISK_FORMATS VST3)
if(APPLE)
  list(APPEND OBELISK_FORMATS AU)
endif()
juce_add_plugin(ObeliskM0
  COMPANY_NAME "Particelle"
  BUNDLE_ID "dev.particelle.obeliskm0"
  PLUGIN_MANUFACTURER_CODE "Prtc"
  PLUGIN_CODE "ObM0"
  FORMATS ${OBELISK_FORMATS}
  PRODUCT_NAME "Obelisk M0"
  IS_SYNTH TRUE
  NEEDS_MIDI_INPUT TRUE
  NEEDS_MIDI_OUTPUT FALSE
  IS_MIDI_EFFECT FALSE
  COPY_PLUGIN_AFTER_BUILD FALSE
)
target_sources(ObeliskM0 PRIVATE PluginProcessor.cpp PluginProcessor.h)
target_include_directories(ObeliskM0 PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../../obelisk-m0/include")
target_compile_definitions(ObeliskM0 PRIVATE JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_REPORT_APP_USAGE=0)
target_link_libraries(ObeliskM0 PRIVATE obelisk_m0_rust juce::juce_audio_utils)
if(APPLE)
  target_link_libraries(ObeliskM0 PRIVATE "-framework Security" "-framework SystemConfiguration")
endif()

include(CTest)
if(BUILD_TESTING)
  juce_add_console_app(ObeliskM0Smoke PRODUCT_NAME "Obelisk M0 Smoke")
  target_sources(ObeliskM0Smoke PRIVATE Smoke.cpp PluginProcessor.cpp PluginProcessor.h)
  target_include_directories(ObeliskM0Smoke PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../../obelisk-m0/include")
  target_compile_definitions(ObeliskM0Smoke PRIVATE JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_REPORT_APP_USAGE=0)
  target_link_libraries(ObeliskM0Smoke PRIVATE obelisk_m0_rust juce::juce_audio_utils)
  if(APPLE)
    target_link_libraries(ObeliskM0Smoke PRIVATE "-framework Security" "-framework SystemConfiguration")
  endif()
  add_test(NAME obelisk_plugin_smoke COMMAND ObeliskM0Smoke)
endif()
