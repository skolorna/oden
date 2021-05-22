import { DateTime } from "luxon";
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

	test("list schools", async () => {
		const response = await app.inject({
			method: "GET",
			url: "/providers/sodexo/schools",
		});

		expect(response.statusCode).toBe(200);
		expect(response.json<School[]>().length).toBeGreaterThan(0);
	});

	test("query school", async () => {
		const schoolResponse = await app.inject({
			method: "GET",
			url: "/providers/sodexo/schools/2ae66740-672e-4183-ab2d-ac1e00b66a5f",
		});

		expect(schoolResponse.statusCode).toBe(200);

		const notFoundResponse = await app.inject({
			method: "GET",
			url: "/providers/sodexo/schools/bruh",
		});

		expect(notFoundResponse.statusCode).toBe(404);
	});

	describe("school menu", () => {
		it("should not accept invalid timestamps", async () => {
			const nonISOResponse = await app.inject({
				method: "GET",
				url: "/providers/skolmaten/schools/85957002/menu",
				query: {
					first: "invalid-iso8601",
				},
			});

			expect(nonISOResponse.statusCode).toBe(422);

			const wrongTimesResponse = await app.inject({
				method: "GET",
				url: "/providers/skolmaten/schools/85957002/menu",
				query: {
					first: DateTime.utc(2021).toISO(),
					last: DateTime.utc(2020).toISO(),
				},
			});

			expect(wrongTimesResponse.statusCode).toBe(400);
		});

		it("should work without any parameters", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/providers/skolmaten/schools/85957002/menu",
			});

			expect(response.statusCode).toBe(200);
		});
	});
});
