import { $ } from "bun";
import { parseArgs } from "util";
import { parseWebStream } from "music-metadata";
import { exit } from "process";

const { values: { endpoint, pathTranslator } } = parseArgs({
    args: Bun.argv,
    options: {
        endpoint: {
            type: "string",
            short: "e",
            default: "localhost:8080",
        },
        pathTranslator: {
            type: "string",
            short: "t",
            default: "",
        },
    },
    allowPositionals: true,
    strict: true,
});

if (!URL.canParse(`http://${endpoint}/cover`)) {
    console.log("could not parse configured endpoint url");
    exit(1);
}

console.log(`connecting to ${endpoint}`);

async function translatePath(path: string): Promise<string> {
    if (!pathTranslator) return path;

    return await $`${pathTranslator} ${path}`
        .nothrow()
        .quiet()
        .text();
}

async function getCover(path: string): Promise<Uint8Array | null> {
    const file = Bun.file(await translatePath(path));

    if (!await file.exists()) {
        console.log(`error: translated file "${await translatePath(path)}" for ${path} does not exist`);
        return null;
    }

    const metadata = await parseWebStream(file.stream());

    const picture = metadata.common.picture?.reduce((acc, val) => {
        if (!acc) return val;
        if (acc.data.length < val.data.length) return val;
        return acc;
    });

    if (!picture) {
        console.log(`error: file ${path} has no picture`);
        return null;
    }

    return picture.data;
}

while (true) {
    const ws = new WebSocket(`ws://${endpoint}/cover`);

    ws.addEventListener("message", async msg => {
        if (!msg.data || typeof msg.data !== "string")
            return;

        const path = msg.data;
        console.log(`loading cover for ${path}`);

        const coverData = await getCover(path);
        if (!coverData) {
            console.log(`loading of ${path} failed`);
            return;
        }

        const url = URL.parse(`http://${endpoint}/cover`)!;
        url.searchParams.set("path", path);

        await fetch(url, {
            method: "POST",
            body: coverData,
        });
    });

    await new Promise(resolve => ws.addEventListener("close", resolve));
    console.log("connection failed, retrying in 30s");
    await Bun.sleep(30 * 1000);
}

