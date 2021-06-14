import { fetchRetry } from "../../src/utils/fetch-retry";

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
});
