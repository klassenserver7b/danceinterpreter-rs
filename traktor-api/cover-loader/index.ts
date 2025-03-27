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
            default: "http://localhost:8080/",
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

if (!URL.canParse(`${endpoint}cover`)) {
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

async function getCover(path: string): Promise<Uint8Array> {
    const file = Bun.file(await translatePath(path));

    if (!await file.exists()) {
        console.log(`error: translated file "${await translatePath(path)}" for ${path} does not exist`);
        return Uint8Array.of();
    }

    const metadata = await parseWebStream(file.stream());

    const picture = metadata.common.picture?.reduce((acc, val) => {
        if (!acc) return val;
        if (acc.data.length < val.data.length) return val;
        return acc;
    });

    if (!picture) {
        console.log(`error: file ${path} has no picture`);
        return Uint8Array.of();
    }

    return picture.data;
}

async function requestLoadCover() {
    console.log("requesting cover to load");

    const response = await fetch(`${endpoint}cover`);
    const path = await response.text();

    if (!path) {
        console.log("no cover was needed");
        return;
    }

    const coverData = await getCover(path);

    const url = URL.parse(`${endpoint}cover`)!;
    url.searchParams.set("path", path);

    await fetch(url, {
        method: "POST",
        body: coverData,
    });
}

while (true) {
    try {
        await requestLoadCover();
    } catch {
        console.log("request failed, retrying in 5s");
        await Bun.sleep(5000);
    }
}

