import { ValidatedRequest, SearchParams } from "../types.ts";
import { REQUEST_URL_REGEX } from "../constants.ts";

export function validateRequestPath(path: string): ValidatedRequest {
	const requestParts = path.match(REQUEST_URL_REGEX);
	const groups = requestParts?.groups;
	if (requestParts == null || groups == null) {
		return {
			valid: false,
			message: "Invalid path.",
		};
	}

	const webhookId = groups.webhook_id;
	if (webhookId == null) {
		return {
			valid: false,
			message: "Missing webhook id.",
		};
	}

	const webhookToken = groups.webhook_token;
	if (webhookToken == null) {
		return {
			valid: false,
			message: "Missing webhook token.",
		};
	}

	let searchParams: SearchParams = {};
	const searchParamsString = groups.params;
	if (searchParamsString != null) {
		const parsedSearchParams = new URLSearchParams(searchParamsString);
		searchParams = {
			wait: parsedSearchParams.get("wait") === "true",
			threadId: parsedSearchParams.get("thread_id") ?? undefined,
		};
	}

	return {
		valid: true,
		webhookId,
		webhookToken,
		searchParams,
	};
}
