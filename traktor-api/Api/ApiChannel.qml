import CSI 1.0
import QtQuick 2.0
import "ApiClient.js" as ApiClient

Item {
    property int index: 0
    readonly property string apiId: `channel${index}`

    AppProperty { id: propCue; path: `app.traktor.mixer.channels.${index + 1}.cue`; onValueChanged: updateState() }
    AppProperty { id: propVolume; path: `app.traktor.mixer.channels.${index + 1}.volume`; onValueChanged: updateState() }
    AppProperty { id: propXFaderLeft; path: `app.traktor.mixer.channels.${index + 1}.xfader_assign.left`; onValueChanged: updateState() }
    AppProperty { id: propXFaderRight; path: `app.traktor.mixer.channels.${index + 1}.xfader_assign.right`; onValueChanged: updateState() }

    function updateState() {
        ApiClient.sendUpdate(apiId, {
            cue: propCue.value,
            volume: propVolume.value,
            xFaderLeft: propXFaderLeft.value,
            xFaderRight: propXFaderRight.value,
        });
    }

    Component.onCompleted: updateState();
}
