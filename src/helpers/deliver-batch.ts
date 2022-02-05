import { DISCORD_WEBHOOK_ENDPOINT } from "../constants.ts";
import { mergeBatchBody } from "./merge-batch-body.ts";
import { RequestBatch } from "../types.ts";

export async function deliverBatch(batch: RequestBatch): Promise<void> {
	const requestBody = mergeBatchBody(batch.payloads);
	const requestOptions: RequestInit = {
		method: "POST",
		body: JSON.stringify(requestBody),
		headers: {
			"Content-Type": "application/json",
		},
	};

	const endpoint = `${DISCORD_WEBHOOK_ENDPOINT}/${batch.webhookId}/${batch.webhookToken}`;
	const discordResponse = await fetch(endpoint, requestOptions);
	const headers = new Headers(discordResponse.headers);
	headers.append("X-Batch-Id", batch.batchId);
	headers.append("X-Batch-Size", batch.payloads.length.toString());
	headers.append("X-Batch-Created", batch.created.toISOString());
	const response = new Response(discordResponse.body, {
		status: discordResponse.status,
		statusText: discordResponse.statusText,
		headers: headers,
	});

	batch.reply(response);
}
