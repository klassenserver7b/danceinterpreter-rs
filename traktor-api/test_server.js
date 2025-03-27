const MAX_QUEUE = 20;
const DEBUG_LOGGING = false;

let sessionId = Bun.randomUUIDv7();
let baseTime = 0;
let state = null;
let queue = [];

let requiredImages = [];
let loadedImages = [];
let coverPing = null;

const resetConnection = () => {
    state = null;
    queue = [];

    requiredImages = [];
    loadedImages = [];
    if (coverPing) coverPing();

    sessionId = Bun.randomUUIDv7();
};

const onUpdateState = () => {
    requiredImages = [
        state.deck0content.filePath,
        state.deck1content.filePath,
        state.deck2content.filePath,
        state.deck3content.filePath,
    ].filter(i => i);
    loadedImages = loadedImages.filter(i => requiredImages.includes(i));
    if (coverPing) coverPing();

    console.log(state);
};

const createCoverPing = () => new Promise(resolve => {
    coverPing = () => {
        coverPing = null;
        resolve();
    };
});

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
            GET: async _ => {
                do {
                    const neededImages = requiredImages.filter(i => !loadedImages.includes(i));

                    if (neededImages.length > 0) {
                        return new Response(neededImages[0]);
                    }

                    await createCoverPing();
                } while (true);
            },
            POST: async req => {
                const { searchParams } = URL.parse(req.url);
                const path = searchParams.get("path");
                if (!path)
                    return new Response("'path' query param missing", { status: 400 });

                const data = await req.blob();
                loadedImages.push(path);

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

        return new Response("Not Found", { status: 404 });
    },
});

console.log(`Listening on ${server.url}`);
