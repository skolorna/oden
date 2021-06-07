import { fetchRetry } from "../../../utils/fetch-retry";

export interface SkolmatenRequestOptions {
	path: string;
}

const performSkolmatenRequest = async <T>(path: string): Promise<T> => {
	const url = `https://skolmaten.se/api/4${path}`;

	const res = await fetchRetry(url, {
		headers: {
			"API-Version": "4.0",
			"Client-Token": "web",
			"Client-Version-Token": "web",
			Locale: "sv_SE",
			"User-Agent": `Block me, I dare you!`,
		},
	});

	const data: T = await res.json();

	return data;
};

export default performSkolmatenRequest;
