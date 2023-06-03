cmake_minimum_required(VERSION 3.10)

if(CMAKE_BUILD_TYPE STREQUAL "Release")
	set(NVIM_RUST_BUILD_TYPE_FLAG "--release")
	set(NVIM_RUST_BUILD_TYPE "release")
else()
	set(NVIM_RUST_BUILD_TYPE "debug")
endif()

if(ANDROID)
	if(ANDROID_ABI STREQUAL "armeabi-v7a")
		set(ANDROID_TARGET "armv7-linux-androideabi")
		set(ANDROID_ARCH_SHORT "arm")
	elseif(ANDROID_ABI STREQUAL "arm64-v8a")
		set(ANDROID_TARGET "aarch64-linux-android")
		set(ANDROID_ARCH_SHORT "aarch64")
	elseif(ANDROID_ABI STREQUAL "x86")
		set(ANDROID_TARGET "i686-linux-android")
		set(ANDROID_ARCH_SHORT "i386")
	elseif(ANDROID_ABI STREQUAL "x86_64")
		set(ANDROID_TARGET "x86_64-linux-android")
		set(ANDROID_ARCH_SHORT "x86_64")
	endif()

	set(NVIM_RUST_BUILD_TARGET "--target=${ANDROID_TARGET}")
endif()

if(WIN32)
	set(NVIM_RUST_TARGET_PREFIX
		${CMAKE_CURRENT_SOURCE_DIR}/target/${NVIM_RUST_BUILD_TYPE}/)
	set(NVIM_RUST_TARGET_POSTFIX .lib)
elseif(ANDROID)
	set(NVIM_RUST_TARGET_PREFIX
		${CMAKE_CURRENT_SOURCE_DIR}/target/${ANDROID_TARGET}/${NVIM_RUST_BUILD_TYPE}/lib)
	set(NVIM_RUST_TARGET_POSTFIX .a)
else()
	set(NVIM_RUST_TARGET_PREFIX
		${CMAKE_CURRENT_SOURCE_DIR}/target/${NVIM_RUST_BUILD_TYPE}/lib)
	set(NVIM_RUST_TARGET_POSTFIX .a)
endif()

set(NVIM_RUST_CRATES "nvim-wasm")
set(NVIM_RUST_BUILD_PRODUCTS "")
foreach(crate_name ${NVIM_RUST_CRATES})
    string(REPLACE "-" "_" crate_name ${crate_name})
    list(APPEND NVIM_RUST_BUILD_PRODUCTS "${NVIM_RUST_TARGET_PREFIX}${crate_name}${NVIM_RUST_TARGET_POSTFIX}")
endforeach()

if(ANDROID)
	# Rust attempts to use libgcc.a on NDK versions r23-beta3 and up
	# but it has been replaced with libunwind.a (rust-lang/rust#85806)
	file(WRITE ${CMAKE_BINARY_DIR}/libgcc.a "INPUT(-lunwind)")
	# The version of the clang compiler is part of the libunwind.a path
	file(STRINGS ${ANDROID_TOOLCHAIN_ROOT}/AndroidVersion.txt CLANG_VERSION_FILE)
	list(GET CLANG_VERSION_FILE 0 CLANG_VERSION)

	# Some crates use the compiler directly, environment variables
	# are set to make them use the Android compiler
	set(NVIM_RUST_PREBUILD_COMMAND ${CMAKE_COMMAND} -E env
	CC=${ANDROID_TOOLCHAIN_ROOT}/bin/clang
	AR=${ANDROID_TOOLCHAIN_ROOT}/bin/llvm-ar
	"RUSTFLAGS=-L ${CMAKE_SYSROOT}/usr/lib/${ANDROID_TARGET}/${ANDROID_NATIVE_API_LEVEL} \
		-L ${ANDROID_TOOLCHAIN_ROOT}/lib64/clang/${CLANG_VERSION}/lib/linux/${ANDROID_ARCH_SHORT} \
		-L ${CMAKE_BINARY_DIR} -C linker=${ANDROID_TOOLCHAIN_ROOT}/bin/ld")
endif()

include(ExternalProject)
ExternalProject_Add(
	nvim-rs-crates
	DOWNLOAD_COMMAND ""
	CONFIGURE_COMMAND ""
	INSTALL_COMMAND "${NVIM_RUST_INSTALL_COMMAND}"
	BUILD_COMMAND ${NVIM_RUST_PREBUILD_COMMAND} cargo build ${NVIM_RUST_BUILD_TYPE_FLAG} ${NVIM_RUST_BUILD_TARGET}
	BINARY_DIR ${CMAKE_CURRENT_SOURCE_DIR}
	BUILD_ALWAYS ON
	BUILD_BYPRODUCTS ${NVIM_RUST_BUILD_PRODUCTS})
add_library(nvim-rs INTERFACE)
add_dependencies(nvim-rs nvim-rs-crates)

if(WIN32)
        target_compile_options(nvim-rs INTERFACE -DWASM_API_EXTERN= -DWASI_API_EXTERN=)
	target_link_libraries(nvim-rs INTERFACE ${NVIM_RUST_BUILD_PRODUCTS}
                ws2_32 advapi32 userenv ntdll shell32 ole32 bcrypt)
elseif(APPLE OR ANDROID)
	target_link_libraries(nvim-rs INTERFACE ${NVIM_RUST_BUILD_PRODUCTS})
else()
	target_link_libraries(nvim-rs INTERFACE ${NVIM_RUST_BUILD_PRODUCTS}
                pthread dl m)
endif()

target_include_directories(nvim-rs INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)

# vim: noet:ts=8:sw=8
