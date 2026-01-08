const MAX_QUEUE = 20;
const DEBUG_LOGGING = false;

let sessionId = Bun.randomUUIDv7();
let baseTime = 0;
let state = null;
let queue = [];

let requiredImages = [];
let loadedImages = [];
let sentImages = [];

const resetConnection = () => {
    state = null;
    queue = [];

    requiredImages = [];
    loadedImages = [];
    sentImages = [];

    sessionId = Bun.randomUUIDv7();
};

const getNeededImages = () => requiredImages.filter(i => !loadedImages.includes(i));

const onUpdateState = () => {
    requiredImages = [
        state.deck0content.filePath,
        state.deck1content.filePath,
        state.deck2content.filePath,
        state.deck3content.filePath,
    ].filter(i => i);
    loadedImages = loadedImages.filter(i => requiredImages.includes(i));
    onChangeNeededImages();

    console.log(state);
};

const onChangeNeededImages = () => {
    const neededImages = getNeededImages();
    const newImages = neededImages.filter(i => !sentImages.includes(i));

    if (newImages.length > 0)
        console.log("needed images changed");

    for (const img of newImages)
        server.publish("cover", img);

    sentImages = neededImages;
};

const server = Bun.serve({
    port: 8080,
    routes: {
        "/connect": () => new Response(JSON.stringify({
            sessionId,
            debugLogging: DEBUG_LOGGING,
        })),
        "/init": {
            POST: async req => {
                const data = await req.json();

                if (data.sessionId === sessionId) {
                    baseTime = data.timestamp;
                    state = data.state;

                    for (const update in queue)
                        Object.apply(state, update);
                    queue = [];

                    onUpdateState();
                }

                return new Response(sessionId);
            },
        },
        "/update/:id": {
            POST: async req => {
                const data = await req.json();

                if (data.sessionId === sessionId) {
                    if (!state) {
                        queue.push(data.state);

                        if (queue.length > MAX_QUEUE)
                            resetConnection();
                    } else {
                        state[req.params.id] = data.state;
                        onUpdateState();
                    }
                }

                return new Response(sessionId);
            },
        },
        "/cover": {
            GET: async req => {
                if (server.upgrade(req))
                    return;

                return new Response("expected websocket connection", {status: 400});
            },
            POST: async req => {
                const {searchParams} = URL.parse(req.url);
                const path = searchParams.get("path");
                if (!path)
                    return new Response("'path' query param missing", {status: 400});

                const data = await req.blob();
                loadedImages.push(path);

                console.log(`image "${path}" received`);

                const file = Bun.file(`./${Bun.randomUUIDv7()}.bin`);
                await Bun.write(file, data);
                return new Response("OK");
            },
        },
        "/log": {
            POST: async req => {
                if (req.body) console.log(`[log] ${await req.text()}`);
                return new Response("OK");
            }
        },
        "/*": async req => {
            console.log(`${req.method}\t${req.url}`);
            if (req.body) console.log(await req.text());

            return new Response("OK");
        },
    },

    async fetch(req) {
        console.log(`${req.method}\t${req.url}`);
        if (req.body) console.log(await req.text());

        return new Response("Not Found", {status: 404});
    },

    websocket: {
        open(ws) {
            console.log("web socket connected");

            for (const img of sentImages)
                ws.send(img);

            ws.subscribe("cover");
        },
        close() {
            console.log("web socket disconnected");
        },
    },
});

console.log(`Listening on ${server.url}`);
