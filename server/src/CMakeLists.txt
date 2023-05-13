
qt_add_executable(mini_winserver
    main.cpp
    winserver.h
    winserver.cpp
    winclient.h
    winclient.cpp
    winclientview.h
    winclientview.cpp
    wineventinterceptor.h
    wineventinterceptor.cpp)

qt_add_qml_module(mini_winserver
    URI org.burbokop.mini_winserver
    VERSION 1.0
    QML_FILES Main.qml)

set_target_properties(mini_winserver PROPERTIES
    MACOSX_BUNDLE_GUI_IDENTIFIER my.example.com
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
    MACOSX_BUNDLE TRUE
    WIN32_EXECUTABLE TRUE)

target_link_libraries(mini_winserver
    PRIVATE Qt6::Quick)

install(TARGETS mini_winserver
    BUNDLE DESTINATION .
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR})
