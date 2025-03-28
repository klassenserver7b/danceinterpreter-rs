pragma Singleton

import QtQuick 2.0

QtObject {
    property var sessionId: ""
    property var debugLogging: false
    property var isConnecting: false
    property var state: ({})
}
