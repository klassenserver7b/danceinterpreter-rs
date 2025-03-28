import CSI 1.0
import QtQuick 2.0
import "ApiClient.js" as ApiClient

Item {
    readonly property string apiId: "mixer"

    // @formatter:off
    AppProperty { id: propXFaderAdjust; path: "app.traktor.mixer.xfader.adjust"; onValueChanged: updateState() }
    AppProperty { id: propMasterVolume; path: "app.traktor.mixer.master_volume"; onValueChanged: updateState() }
    AppProperty { id: propCueVolume; path: "app.traktor.mixer.cue.volume"; onValueChanged: updateState() }
    AppProperty { id: propCueMix; path: "app.traktor.mixer.cue.mix"; onValueChanged: updateState() }
    AppProperty { id: propMicVolume; path: "app.traktor.mixer.mic_volume"; onValueChanged: updateState() }
    // @formatter:on

    function updateState() {
        ApiClient.sendUpdate(apiId, {
            xFader: propXFaderAdjust.value,
            masterVolume: propMasterVolume.value,
            cueVolume: propCueVolume.value,
            cueMix: propCueMix.value,
            micVolume: propMicVolume.value,
        });
    }

    Component.onCompleted: updateState();
}
