import build from "../src/app";
import { ProviderInfo } from "../src/providers/types";
import { School } from "../src/types";

describe("main application tests", () => {
	const app = build();

	test("health check", async () => {
		const response = await app.inject({
			method: "GET",
			url: "/health",
		});

		expect(response.statusCode).toBe(200);
	});

	test("provider information", async () => {
		const listResponse = await app.inject({
			method: "GET",
			url: "/providers",
		});

		expect(listResponse.statusCode).toBe(200);

		const providers = listResponse.json<ProviderInfo[]>();

		await Promise.all(
			providers.map(async (provider) => {
				const response = await app.inject({
					method: "GET",
					url: `/providers/${provider.id}`,
				});

				expect(response.statusCode).toBe(200);
				expect(response.json()).toEqual(provider);
			}),
		);
	});

	test("provider schools", async () => {
		const response = await app.inject({
			method: "GET",
			url: "/providers/mashie/schools",
		});

		expect(response.statusCode).toBe(200);
		expect(response.json<School[]>().length).toBeGreaterThan(0);
	});
});
