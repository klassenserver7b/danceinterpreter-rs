import CSI 1.0
import QtQuick 2.0
import "ApiClient.js" as ApiClient

Item {
    property int index: 0
    readonly property string contentApiId: `deck${index}content`
    readonly property string playStateApiId: `deck${index}playstate`

    property bool initialized: false

    property bool prevIsLoaded: false
    property bool prevIsLoadedSignal: false

    property real remoteTimestamp: 0
    property real remotePosition: 0
    property real remoteSpeed: 0

    AppProperty { id: propIsLoaded; path: `app.traktor.decks.${index + 1}.is_loaded`; onValueChanged: updateContent(false) }
    AppProperty { id: propIsLoadedSignal; path: `app.traktor.decks.${index + 1}.is_loaded_signal`; onValueChanged: updateContent(false) }

    AppProperty { id: propTitle; path: `app.traktor.decks.${index + 1}.content.title`; onValueChanged: updateContentProp() }
    AppProperty { id: propArtist; path: `app.traktor.decks.${index + 1}.content.artist`; onValueChanged: updateContentProp() }
    AppProperty { id: propAlbum; path: `app.traktor.decks.${index + 1}.content.album`; onValueChanged: updateContentProp() }
    AppProperty { id: propGenre; path: `app.traktor.decks.${index + 1}.content.genre`; onValueChanged: updateContentProp() }
    AppProperty { id: propComment; path: `app.traktor.decks.${index + 1}.content.comment`; onValueChanged: updateContentProp() }
    AppProperty { id: propComment2; path: `app.traktor.decks.${index + 1}.content.comment2`; onValueChanged: updateContentProp() }
    AppProperty { id: propLabel; path: `app.traktor.decks.${index + 1}.content.label`; onValueChanged: updateContentProp() }
    AppProperty { id: propKey; path: `app.traktor.decks.${index + 1}.content.musical_key`; onValueChanged: updateContentProp() }
    AppProperty { id: propFilePath; path: `app.traktor.decks.${index + 1}.track.content.file_path`; onValueChanged: updateContentProp() }
    AppProperty { id: propTrackLength; path: `app.traktor.decks.${index + 1}.track.content.track_length`; onValueChanged: updateContentProp() }
    AppProperty { id: propBpm; path: `app.traktor.decks.${index + 1}.tempo.base_bpm`; onValueChanged: updateContentProp() }

    AppProperty {id: propPlayheadPosition; path: `app.traktor.decks.${index + 1}.track.player.playhead_position`; onValueChanged: checkPlayState() }
    AppProperty {id: propEffectiveTempo; path: `app.traktor.decks.${index + 1}.track.player.effective_tempo`; onValueChanged: updatePlayState() }

    function checkPlayState() {
        const guessedPosition = (Date.now() - remoteTimestamp) / 1000 * remoteSpeed + remotePosition;
        const actualPosition = propPlayheadPosition.value;

        const error = Math.abs(actualPosition - guessedPosition);
        const speedError = Math.abs(remoteSpeed - propEffectiveTempo.value);

        ApiClient.log(`play state error ${error.toFixed(10)} speed error ${speedError.toFixed(10)}`);

        if (error > 0.01 && speedError < 1e-8 && initialized) {
            ApiClient.log(`---------------- SEEK DETECTED ON DECK ${index} ----------------`);
            updatePlayState();
        }
    }

    function updatePlayState() {
        remoteTimestamp = Date.now();
        remotePosition = propPlayheadPosition.value;
        remoteSpeed = propEffectiveTempo.value;

        ApiClient.sendUpdate(playStateApiId, {
            timestamp: remoteTimestamp,
            position: remotePosition,
            speed: remoteSpeed,
        });
    }

    function updateContentProp() {
        if (!propIsLoaded.value || !initialized) return;
        updateContent(true);
    }

    function updateContent(force) {
        const updateRequired = force
            || (prevIsLoaded && !propIsLoaded.value)
            || (!prevIsLoadedSignal && propIsLoadedSignal.value);

        prevIsLoaded = propIsLoaded.value;
        prevIsLoadedSignal = propIsLoadedSignal.value;
        if (!updateRequired) return;

        const isLoaded = propIsLoaded.value;

        ApiClient.sendUpdate(contentApiId, {
            isLoaded,

            title: isLoaded ? propTitle.value : "",
            artist: isLoaded ? propArtist.value : "",
            album: isLoaded ? propAlbum.value : "",
            genre: isLoaded ? propGenre.value : "",
            comment: isLoaded ? propComment.value : "",
            comment2: isLoaded ? propComment2.value : "",
            label: isLoaded ? propLabel.value : "",

            key: isLoaded ? propKey.value : "",
            filePath: isLoaded ? propFilePath.value : "",
            trackLength: isLoaded ? propTrackLength.value : 0,
            bpm: isLoaded ? propBpm.value : 0,
        });
    }

    function initialize() {
        updateContent(true);
        updatePlayState();
        initialized = true;
    }

     Component.onCompleted: initialize();
}
