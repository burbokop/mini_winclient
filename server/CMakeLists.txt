cmake_minimum_required(VERSION 3.16)

project(mini_winserver VERSION 0.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.2 REQUIRED COMPONENTS Quick)

qt_standard_project_setup()

add_subdirectory(src)

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/config/systemd/mini-winserver.service.in
               ${CMAKE_CURRENT_BINARY_DIR}/config/systemd/mini-winserver.service)

install(FILES ${CMAKE_CURRENT_BINARY_DIR}/config/systemd/mini-winserver.service
    DESTINATION ${CMAKE_INSTALL_PREFIX}/mini_winserver/config/systemd)

# Package Generator  #######################################################
set(CPACK_PACKAGE_DESCRIPTION ${CMAKE_PROJECT_DESCRIPTION})
set(CPACK_PACKAGE_VERSION_MAJOR ${VERSION_MAJOR})
set(CPACK_PACKAGE_VERSION_MINOR ${VERSION_MINOR})
set(CPACK_PACKAGE_VERSION_PATCH ${VERSION_PATCH})
set(CPACK_PACKAGE_VENDOR "burbokop")
set(CPACK_PACKAGE_CONTACT "burbokop@gmail.com")
set(CPACK_PACKAGE_FILE_NAME "${CMAKE_PROJECT_NAME}-${CMAKE_PROJECT_VERSION}")

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/packaging/debian-control/postinst.in
               ${CMAKE_CURRENT_BINARY_DIR}/packaging/debian-control/postinst)
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/packaging/debian-control/prerm.in
               ${CMAKE_CURRENT_BINARY_DIR}/packaging/debian-control/prerm)
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/packaging/debian-control/postrm.in
               ${CMAKE_CURRENT_BINARY_DIR}/packaging/debian-control/postrm)

set(CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_BINARY_DIR}/packaging/debian-control/postinst;${CMAKE_CURRENT_BINARY_DIR}/packaging/debian-control/prerm;${CMAKE_CURRENT_BINARY_DIR}/packaging/debian-control/postrm")

set(CPACK_DEBIAN_PACKAGE_DEPENDS "
    libqt6core6 (>= ${Qt6_VERSION}+dfsg~),
    libqt6quick6 (>= ${Qt6_VERSION}+dfsg~),
    qml6-module-qtquick-window (>= ${Qt6_VERSION}+dfsg~),
    qml6-module-qtqml-workerscript (>= ${Qt6_VERSION}+dfsg~)")

string(REGEX REPLACE "\n[ ]*" " " CPACK_DEBIAN_PACKAGE_DEPENDS ${CPACK_DEBIAN_PACKAGE_DEPENDS})
string(STRIP ${CPACK_DEBIAN_PACKAGE_DEPENDS} CPACK_DEBIAN_PACKAGE_DEPENDS)

include (CPack)
