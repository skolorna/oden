import { fetchWithUserAgent } from "../../src/menus/fetch-with-ua";

describe("fetch with user agent tests", () => {
	it("should have the correct user agent", async () => {
		const res = await fetchWithUserAgent("http://httpbin.org/headers");

		const data: { headers: Record<string, string> } = await res.json();

		expect(data.headers["User-Agent"]).toBe("Block me, I dare you");
	});
});
