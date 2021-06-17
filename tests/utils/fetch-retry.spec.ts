import { fetchRetry } from "../../src/utils/fetch-retry";

interface HTTPBinHeadersResponse {
	headers: Record<string, string>;
}

describe("fetch retry tests", () => {
	it("should retry", async () => {
		jest.setTimeout(5000);

		await expect(
			fetchRetry("https://httpbin.org/status/500", undefined, { backoff: 100 }),
		).rejects.toThrowErrorMatchingInlineSnapshot(`"http request failed after 3 retries"`);
	});

	it("should skip 404s", async () => {
		const res = await fetchRetry("https://httpbin.org/status/404", undefined, { backoff: 100 });

		expect(res.status).toBe(404);
	});

	it("should work", async () => {
		const res = await fetchRetry("https://httpbin.org/status/200");

		expect(res.status).toBe(200);
	});

	it("should add a user-agent header", async () => {
		const res = await fetchRetry("https://httpbin.org/headers", {
			// It should also respect existing headers:
			headers: {
				"Cache-Control": "no-cache",
				"X-Cool-Header": "indeed",
			},
		});

		const data: HTTPBinHeadersResponse = await res.json();

		expect(data.headers["User-Agent"]).toBe("Block me, I dare you");
		expect(data.headers["Cache-Control"]).toBe("no-cache");
		expect(data.headers["X-Cool-Header"]).toBe("indeed");
	});

	it("should respect custom user-agent", async () => {
		const res = await fetchRetry("https://httpbin.org/headers", {
			headers: {
				"User-Agent": "my custom user agent",
			},
		});

		const data: HTTPBinHeadersResponse = await res.json();

		expect(data.headers["User-Agent"]).toBe("my custom user agent");
	});
});
