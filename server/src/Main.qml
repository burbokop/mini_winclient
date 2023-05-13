import QtQuick
import QtQuick.Window
import QtQuick.Controls
import org.burbokop.mini_winserver

Item {
    id: root

    required property int port;

    Instantiator {
        model: MiniWinServer {
            port: root.port
            verbose: true
        }

        Window {
            id: window

            visible: true
            width: display.width
            height: display.height
            title: display.title

            WinEventInterceptor {
                target: window
                client: display
            }

            WinClientView {
                client: display
            }
        }
    }
}
