import { ValidatedRequest } from "../types.ts";
import { REQUEST_URL_REGEX } from "../constants.ts";

export function validateRequestPath(path: string): ValidatedRequest {
	const requestParts = path.match(REQUEST_URL_REGEX);
	if (requestParts == null || requestParts.groups == null) {
		return {
			valid: false,
			message: "Invalid path.",
		};
	}

	const webhookId = requestParts.groups.webhook_id;
	if (webhookId == null) {
		return {
			valid: false,
			message: "Missing webhook id.",
		};
	}

	const webhookToken = requestParts.groups.webhook_token;
	if (webhookToken == null) {
		return {
			valid: false,
			message: "Missing webhook token.",
		};
	}

	return {
		valid: true,
		webhookId,
		webhookToken,
	};
}
