import build from "../src/app";
import { Menu } from "../src/types";

describe("main application tests", () => {
	const app = build();

	test("health check", async () => {
		const response = await app.inject({
			method: "GET",
			url: "/health",
		});

		expect(response.statusCode).toBe(200);
	});

	test("list menus", async () => {
		const response = await app.inject({
			method: "GET",
			url: "/menus",
		});

		expect(response.statusCode).toBe(200);
		expect(response.json<Menu[]>().length).toBeGreaterThan(5000);
	});

	test("query menu", async () => {
		const schoolResponse = await app.inject({
			method: "GET",
			url: "/menus/sodexo.2ae66740-672e-4183-ab2d-ac1e00b66a5f",
		});

		expect(schoolResponse.statusCode).toBe(200);

		const notFoundResponse = await app.inject({
			method: "GET",
			url: "/menus/sodexo.0",
		});

		expect(notFoundResponse.statusCode).toBe(404);
	});

	describe("menu days", () => {
		it("should not accept invalid timestamps", async () => {
			const nonISOResponse = await app.inject({
				method: "GET",
				url: "/menus/skolmaten.85957002/days",
				query: {
					first: "invalid-iso8601",
				},
			});

			expect(nonISOResponse.statusCode).toBe(422);

			const wrongTimesResponse = await app.inject({
				method: "GET",
				url: "/menus/skolmaten.85957002/days",
				query: {
					first: "2021-01-01",
					last: "2020-01-01",
				},
			});

			expect(wrongTimesResponse.statusCode).toBe(400);
		});

		it("should work without any parameters", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/menus/skolmaten.85957002/days",
			});

			expect(response.statusCode).toBe(200);
		});
	});
});
