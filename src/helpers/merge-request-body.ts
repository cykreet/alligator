import { Payload } from "../types.ts";

export function mergeRequestBody(payloads: Payload[]) {
	const requestPayload: Payload = {};
	for (const payload of payloads) {
		if (payload.content) {
			if (requestPayload.content) {
				requestPayload.content += `\n${payload.content}`;
				continue;
			}

			requestPayload.content = payload.content;
		}

		if (payload.embeds) {
			if (requestPayload.embeds) {
				requestPayload.embeds.push(...payload.embeds);
				continue;
			}

			requestPayload.embeds = payload.embeds;
		}

		if (payload.components) {
			if (requestPayload.components) {
				requestPayload.components.push(...payload.components);
				continue;
			}

			requestPayload.components = payload.components;
		}
	}

	return requestPayload;
}
