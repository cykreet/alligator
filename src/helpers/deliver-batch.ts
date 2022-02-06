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
	const requestURL = new URL(endpoint);
	const requestSearchParams = requestURL.searchParams;
	if (batch.searchParams?.wait) requestSearchParams.append("wait", "true");
	if (batch.searchParams?.threadId) requestSearchParams.append("thread_id", batch.searchParams.threadId);
	const discordResponse = await fetch(requestURL, requestOptions);

	const responseHeaders = new Headers(discordResponse.headers);
	responseHeaders.append("X-Batch-Id", batch.batchId);
	responseHeaders.append("X-Batch-Size", batch.payloads.length.toString());
	responseHeaders.append("X-Batch-Created", batch.created.toISOString());
	const response = new Response(discordResponse.body, {
		status: discordResponse.status,
		statusText: discordResponse.statusText,
		headers: responseHeaders,
	});

	batch.reply(response);
}
