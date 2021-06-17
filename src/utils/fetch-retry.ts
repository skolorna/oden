import { InternalServerError } from "http-errors";
import fetch, { RequestInfo, RequestInit, Response } from "node-fetch";

export function sleep(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export interface FetchRetryOptions {
	/**
	 * Maximum number of attempts to make before throwing an error.
	 */
	maxAttempts?: number;

	/**
	 * Initial backoff period (ms), i.e. how long to wait before attempting again. For every attempt, this value is doubled.
	 */
	backoff?: number;

	/**
	 * Array of HTTP status codes that are rejected.
	 *
	 * @example
	 * [500, 502, 503, 504]
	 */
	retryOn?: number[];
}

/**
 * # `fetch` on steroids
 *
 * `fetch` but it performs the request again if the result is bad (for any sensible status code; 404s are not retried by default).
 *
 * It also sets the `User-Agent` header.
 */
export async function fetchRetry(
	url: RequestInfo,
	init: RequestInit = {},
	{ maxAttempts = 3, backoff = 250, retryOn = [408, 500, 502, 503, 504, 524] }: FetchRetryOptions = {},
): Promise<Response> {
	if (backoff < 0) {
		throw new Error(`backoff must be a non-negative number (got ${backoff}).`);
	}

	let attempts = 0;

	while (attempts < maxAttempts) {
		attempts += 1;

		// eslint-disable-next-line no-await-in-loop
		const res = await fetch(url, {
			...init,
			headers: {
				"User-Agent": "Block me, I dare you",
				...init?.headers,
			},
		});

		if (!retryOn.includes(res.status)) {
			return res;
		}

		// eslint-disable-next-line no-await-in-loop
		await sleep(backoff * attempts);
	}

	throw new InternalServerError(`http request failed after ${attempts} attempts`);
}
