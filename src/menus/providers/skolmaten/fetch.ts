import { URLSearchParams } from "url";
import urljoin from "url-join";
import { fetchRetry } from "../../../utils/fetch-retry";

export interface SkolmatenRequestOptions {
	path: string;
}

export function getSkolmatenUrl(path: string, searchParams?: URLSearchParams): string {
	let queryString = searchParams?.toString() ?? "";

	if (queryString.length > 0) {
		queryString = `?${queryString}`;
	}

	return urljoin("https://skolmaten.se/api/4", path, queryString);
}

export default async function skolmatenFetch<T>(path: string, searchParams?: URLSearchParams): Promise<T> {
	const url = getSkolmatenUrl(path, searchParams);

	const res = await fetchRetry(url, {
		headers: {
			"API-Version": "4.0",
			"Client-Token": "web",
			"Client-Version-Token": "web",
			Locale: "sv_SE",
		},
	});

	const data: T = await res.json();

	return data;
}
