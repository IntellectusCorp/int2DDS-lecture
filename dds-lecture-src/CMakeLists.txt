cmake_minimum_required(VERSION 3.10)
project(int2dds_lecture_labs C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# int2DDS SDK 위치. 기본은 이 폴더의 sdk/ (include · lib · bin).
# 다른 SDK는 -DINT2DDS_SDK_DIR=<sdk>, 폴더 구성이 다르면 -DINT2DDS_INCLUDE_DIR / _LIB_DIR / _BIN_DIR 로 지정한다.
set(INT2DDS_SDK_DIR "${CMAKE_CURRENT_SOURCE_DIR}/sdk" CACHE PATH "int2DDS SDK (include, lib, bin)")
set(INT2DDS_INCLUDE_DIR "${INT2DDS_SDK_DIR}/include" CACHE PATH "int2dds-ffi.h, int2dds_cdr.h")
set(INT2DDS_LIB_DIR "${INT2DDS_SDK_DIR}/lib" CACHE PATH "int2dds_ffi.dll.lib")
set(INT2DDS_BIN_DIR "${INT2DDS_SDK_DIR}/bin" CACHE PATH "int2dds_ffi.dll, int2dds-idl.exe")

# PATH는 보지 않는다: cargo install 등으로 깔린 다른 버전의 int2dds-idl이 DLL과 섞이지 않게
find_program(INT2DDS_IDL int2dds-idl HINTS "${INT2DDS_BIN_DIR}" NO_DEFAULT_PATH)
if(NOT INT2DDS_IDL)
    message(FATAL_ERROR "int2dds-idl not found in ${INT2DDS_BIN_DIR} (-DINT2DDS_IDL=<path>)")
endif()

set(INT2DDS_LIB "${INT2DDS_LIB_DIR}/int2dds_ffi.dll.lib")
set(INT2DDS_DLL "${INT2DDS_BIN_DIR}/int2dds_ffi.dll")
foreach(f "${INT2DDS_INCLUDE_DIR}/int2dds-ffi.h" "${INT2DDS_LIB}" "${INT2DDS_DLL}")
    if(NOT EXISTS "${f}")
        message(FATAL_ERROR "${f} not found (INT2DDS_SDK_DIR=${INT2DDS_SDK_DIR})")
    endif()
endforeach()

set(GEN_DIR "${CMAKE_BINARY_DIR}/gen")
file(MAKE_DIRECTORY "${GEN_DIR}")
include_directories("${INT2DDS_INCLUDE_DIR}" "${CMAKE_CURRENT_SOURCE_DIR}/common" "${GEN_DIR}")
# add_compile_definitions는 3.12부터라 3.10에서도 되는 add_definitions를 쓴다
add_definitions(-DLAB_XML_DIR="${CMAKE_CURRENT_SOURCE_DIR}/lab8_xml")
if(MSVC)
    add_definitions(-D_CRT_SECURE_NO_WARNINGS)
endif()

# IDL → C 헤더 (실습 절차의 int2dds-idl -c gen/x.h x.idl 과 같다). SDK의 int2dds-idl이 바뀌어도 다시 만든다
function(lab_idl name idl)
    add_custom_command(
        OUTPUT "${GEN_DIR}/${name}.h"
        COMMAND ${INT2DDS_IDL} -c "${GEN_DIR}/${name}.h" "${idl}"
        DEPENDS "${idl}" "${INT2DDS_IDL}"
        COMMENT "int2dds-idl -c gen/${name}.h")
    add_custom_target(gen_${name} DEPENDS "${GEN_DIR}/${name}.h")
endfunction()
lab_idl(hello "${CMAKE_CURRENT_SOURCE_DIR}/lab1_hello/hello.idl")
lab_idl(floor "${CMAKE_CURRENT_SOURCE_DIR}/lab2_instance_lifecycle/floor.idl")

function(lab_exe target src gen)
    add_executable(${target} "${src}")
    target_link_libraries(${target} "${INT2DDS_LIB}")
    if(gen)
        add_dependencies(${target} gen_${gen})
    endif()
    add_custom_command(TARGET ${target} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different "${INT2DDS_DLL}" $<TARGET_FILE_DIR:${target}>)
endfunction()

lab_exe(lab1_pub lab1_hello/pub.c hello)
lab_exe(lab1_sub lab1_hello/sub.c hello)
lab_exe(lab2_pub lab2_instance_lifecycle/pub.c floor)
lab_exe(lab2_sub lab2_instance_lifecycle/sub.c floor)
lab_exe(lab3_sub lab3_discovery/sub.c hello)
lab_exe(lab4_pub lab4_partition/pub.c hello)
lab_exe(lab4_sub lab4_partition/sub.c hello)
lab_exe(lab5_sub_read lab5_data_receive/sub_read.c floor)
lab_exe(lab5_sub_listener lab5_data_receive/sub_listener.c floor)
lab_exe(lab5_sub_waitset lab5_data_receive/sub_waitset.c floor)
lab_exe(lab6_pub lab6_qos_compatibility/pub.c hello)
lab_exe(lab6_sub lab6_qos_compatibility/sub.c hello)
lab_exe(lab7_pub lab7_qos_test/pub.c hello)
lab_exe(lab7_sub lab7_qos_test/sub.c hello)
lab_exe(lab8_sub_xml lab8_xml/sub_xml.c "")
lab_exe(lab8_sub_profile lab8_xml/sub_profile.c "")
lab_exe(lab8_sub_config lab8_xml/sub_config.c "")
