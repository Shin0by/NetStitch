cmake_minimum_required(VERSION 3.20)
project(ui_entity_showcase_cpp LANGUAGES CXX)

# This CMake project is intentionally minimal. A copied module can add normal
# include paths or dependencies, but it should keep exporting the same two C ABI
# functions: netstitch_integration_call and netstitch_integration_free.
#
# The target name controls the produced native library:
# Windows: ui_entity_showcase_cpp.dll
# Linux:   libui_entity_showcase_cpp.so
# macOS:   libui_entity_showcase_cpp.dylib
# These filenames must match module.json -> library_paths. If you rename the
# target, update both this file and module.json before rebuilding the archive.
add_library(ui_entity_showcase_cpp SHARED ui_entity_showcase_module.cpp)
target_compile_features(ui_entity_showcase_cpp PRIVATE cxx_std_17)
if(MSVC)
    target_compile_options(ui_entity_showcase_cpp PRIVATE /utf-8)
endif()
