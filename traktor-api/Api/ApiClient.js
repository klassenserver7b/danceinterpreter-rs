var ENDPOINT = "http://localhost:8080/";

function sendUpdate(id, data) {
    log(`${id} update (${JSON.stringify(data)})`);
    ApiState.state[id] = data;

    if (!ApiState.sessionId) {
        tryConnect();
        return;
    }

    sendData(`update/${id}`, {
        sessionId: ApiState.sessionId,
        state: data,
    });
}

function tryConnect() {
    if (ApiState.isConnecting) return;
    ApiState.isConnecting = true;

    var request = new XMLHttpRequest();
    request.onreadystatechange = function() {
        if (request.readyState !== XMLHttpRequest.DONE) return;
        ApiState.isConnecting = false;
        if (!request.responseText) return;

        var responseData = JSON.parse(request.responseText);
        ApiState.sessionId = responseData.sessionId;
        ApiState.debugLogging = responseData.debugLogging;

        log(`connecting to ${ApiState.sessionId}`);
        initConnection();
    };

    request.open("POST", ENDPOINT + "connect");
    request.setRequestHeader("Content-Type", "application/json");
    request.send(JSON.stringify({
        time: Date.now(),
    }));
}

function initConnection() {
    sendData("init", {
        sessionId: ApiState.sessionId,
        state: ApiState.state,
    });
}

function sendData(endpoint, data) {
    var request = new XMLHttpRequest();
    request.onreadystatechange = function() {
        if (request.readyState !== XMLHttpRequest.DONE) return;
        if (!request.responseText) {
            ApiState.sessionId = "";
            ApiState.debugLogging = false;
            return;
        }

        if (ApiState.sessionId !== request.responseText) {
            ApiState.sessionId = "";
            ApiState.debugLogging = false;
            tryConnect();
        }
    };

    request.open("POST", ENDPOINT + endpoint);
    request.setRequestHeader("Content-Type", "application/json");
    request.send(JSON.stringify(data));
}

function log(msg, isImportant) {
    if (!isImportant && !ApiState.debugLogging)
        return;

    var request = new XMLHttpRequest();
    request.open("POST", ENDPOINT + "log");
    request.setRequestHeader("Content-Type", "text/plain;charset=utf-8");
    request.send(msg);
}

