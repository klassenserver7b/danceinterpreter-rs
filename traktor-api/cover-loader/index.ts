import { $ } from "bun";
import { parseArgs } from "util";
import { parseWebStream } from "music-metadata";
import { exit } from "process";
import GUTENMORGEN from "bonjour";

const bonjour = GUTENMORGEN();

// @ts-ignore
const {
	values: { providedEndpoint, pathTranslator }
} = parseArgs({
	args: Bun.argv,
	options: {
		providedEndpoint: {
			type: "string",
			short: "e"
		},
		pathTranslator: {
			type: "string",
			short: "t",
			default: ""
		}
	},
	allowPositionals: true,
	strict: true
});

function discoverMdnsServices(): Promise<string[]> {
	return new Promise((res, rej) => {
		console.log("No endpoint provided, discovering via mDNS...");

		const browser = bonjour.find({ type: "http", protocol: "tcp" }, service => {
			if (service.name !== "traktor-di-webserver") {
				return;
			}
			const servers = service.addresses.map(adress => adress.concat(":", String(service.port)));
			clearInterval(interval);

			browser.stop();
			res(servers);
		});

		let interval = setInterval(() => {
			browser.update();
		}, 5000);
	});
}

async function getEndpoints(): Promise<string[]> {
	if (providedEndpoint) {
		console.log(`Using provided endpoint: ${providedEndpoint}`);
		return [providedEndpoint];
	}

	try {
		const discovered = await discoverMdnsServices();
		console.log(`Discovered services at: ${discovered}`);
		return discovered;
	} catch (error: any) {
		console.error(error.message);
		exit(1);
	}
}

async function translatePath(path: string): Promise<string> {
	if (!pathTranslator) return path;

	return await $`${pathTranslator} ${path}`.nothrow().quiet().text();
}

async function getCover(path: string): Promise<Uint8Array | null> {
	const file = Bun.file(await translatePath(path));

	if (!(await file.exists())) {
		console.log(`Error: Translated file "${await translatePath(path)}" for ${path} does not exist`);
		return null;
	}

	const metadata = await parseWebStream(file.stream());

	const picture = metadata.common.picture?.reduce((acc, val) => {
		if (!acc) return val;
		if (acc.data.length < val.data.length) return val;
		return acc;
	});

	if (!picture) {
		console.log(`Error: File ${path} has no picture`);
		return null;
	}

	return picture.data;
}

function startProxyServer(endpoint: string) {
	return Bun.serve({
		port: 8080,
		hostname: "localhost",
		fetch(req) {
			let url = URL.parse(req.url);

			if (!url) {
				return new Response("Bro i actually don't know what you did, but you seriously fucker up!", {
					status: 418
				});
			}

			url.host = endpoint;
			let newReq = new Request(url.toString(), req);
			return fetch(newReq);
		}
	});
}

while (true) {
	// Get the endpoint (either provided or discovered)
	let endpoints = await getEndpoints();
	endpoints = endpoints.filter(e => URL.canParse(`http://${e}/cover`));

	if (endpoints.length === 0) {
		console.log("Could not parse configured endpoint url");
		exit(1);
	}

	for (let endpoint of endpoints) {
		console.log(`Connecting to ${endpoint}`);

		const ws = new WebSocket(`ws://${endpoint}/cover`);
		let connected = false;

		let server: Bun.Server | null = null;

		ws.addEventListener("open", () => {
			connected = true;
			console.log(`Successfully connected to ${endpoint}`);
			server = startProxyServer(endpoint);
		});

		ws.addEventListener("message", async msg => {
			if (!msg.data || typeof msg.data !== "string") return;

			const path = msg.data;
			console.log(`Loading cover for ${path}`);

			const coverData = await getCover(path);
			if (!coverData) {
				console.log(`Loading of ${path} failed`);
				return;
			}

			const url = URL.parse(`http://${endpoint}/cover`)!;
			url.searchParams.set("path", path);

			await fetch(url, {
				method: "POST",
				body: coverData
			});
		});

		await new Promise(resolve => ws.addEventListener("close", resolve));
		console.log(`Cancelled connection to ${endpoint}`);

		if (server) server.stop();

		if (connected) break;
	}

	console.log("Connection failed, retrying in 30s");
	await Bun.sleep(30 * 1000);
}
