import fetch from "node-fetch";

const performSkolmatenRequest = async <T>(path: string): Promise<T> => {
	const url = `https://skolmaten.se/api/4${path}`;

	const data: T = await fetch(url, {
		headers: {
			"API-Version": "4.0",
			"Client-Token": "web",
			"Client-Version-Token": "web",
			Locale: "sv_SE",
		},
	}).then((res) => res.json());

	return data;
};

export default performSkolmatenRequest;
