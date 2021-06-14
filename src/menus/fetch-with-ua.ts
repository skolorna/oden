import { fetchRetry } from "../utils/fetch-retry";

/**
 * Make a request and blend in with the others.
 */
export const fetchWithUserAgent: typeof fetchRetry = (url, init, fetchRetryOptions) => {
	return fetchRetry(
		url,
		{
			headers: {
				"User-Agent": "Block me, I dare you",
				...init?.headers,
			},
			...init,
		},
		fetchRetryOptions,
	);
};
