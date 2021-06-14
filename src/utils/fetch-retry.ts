import { InternalServerError } from "http-errors";
import fetch, { RequestInfo, RequestInit, Response } from "node-fetch";

export function timeout(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export interface FetchRetryOptions {
	maxRetries?: number;
	backoff?: number;
	retryOn?: number[];
	retriesMade?: number;
}

/**
 * # `fetch` on steroids
 *
 * `fetch` but it performs the request again if the result is bad (for any sensible status code; 404s are not retried by default).
 *
 */
export async function fetchRetry(
	url: RequestInfo,
	init: RequestInit = {},
	{ maxRetries = 3, backoff = 250, retryOn = [408, 500, 502, 503, 504, 524], retriesMade = 0 }: FetchRetryOptions = {},
): Promise<Response> {
	const res = await fetch(url, init);

	if (!retryOn.includes(res.status)) {
		return res;
	}

	if (retriesMade < maxRetries) {
		await timeout(backoff);

		return fetchRetry(url, init, {
			maxRetries,
			backoff: backoff * 2,
			retriesMade: retriesMade + 1,
			retryOn,
		});
	}

	throw new InternalServerError(`http request failed after ${retriesMade} retries`);
}
