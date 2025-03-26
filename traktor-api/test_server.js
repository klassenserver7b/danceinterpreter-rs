const MAX_QUEUE = 20;
const DEBUG_LOGGING = false;

let sessionId = Bun.randomUUIDv7();
let baseTime = 0;
let state = null;
let queue = [];

const resetConnection = () => {
    state = null;
    queue = [];
    sessionId = Bun.randomUUIDv7();
};

const onUpdateState = () => {
    console.log(state);
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
