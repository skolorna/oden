import fetch from "node-fetch";

const API_TOKEN = "j44i0zuqo8izmlwg5blh";

const performSkolmatenRequest = async <T>(path: string): Promise<T> => {
	const url = `https://skolmaten.se/api/3${path}`;

	const data: T = await fetch(url, {
		headers: {
			client: API_TOKEN,
		},
	}).then((res) => res.json());

	return data;
};

export default performSkolmatenRequest;
