import { WebhookPayload } from "../../deps.ts";

export function mergeRequestBody(payloads: WebhookPayload[]) {
	const requestPayload: WebhookPayload = {};
	for (let i = 0; i < payloads.length; i++) {
		const payload = payloads[i];
		if (payload.content) {
			if (requestPayload.content) {
				requestPayload.content += `\n${payload.content}`;
			} else {
				requestPayload.content = payload.content;
			}
		}

		if (payload.embeds) {
			if (requestPayload.embeds) {
				requestPayload.embeds.push(...payload.embeds);
			} else {
				requestPayload.embeds = payload.embeds;
			}
		}

		if (payload.components) {
			if (requestPayload.components) {
				requestPayload.components.push(...payload.components);
			} else {
				requestPayload.components = payload.components;
			}
		}

		if (payload.attachments) {
			if (requestPayload.attachments) {
				requestPayload.attachments.push(...payload.attachments);
			} else {
				requestPayload.attachments = payload.attachments;
			}
		}

		const initialPayload = i === 0;
		if (!initialPayload) continue;
		if (payload.username) requestPayload.username = payload.username;
		if (payload.avatar_url) requestPayload.avatar_url = payload.avatar_url;
		if (payload.allowed_mentions) requestPayload.allowed_mentions = payload.allowed_mentions;
		if (payload.tts) requestPayload.tts = payload.tts;
	}

	return requestPayload;
}
