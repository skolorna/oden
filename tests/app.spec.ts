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
		expect(response.headers["cache-control"]).toBe("no-cache");
	});

	test("list menus", async () => {
		jest.setTimeout(20000);

		const response = await app.inject({
			method: "GET",
			url: "/menus",
		});

		expect(response.statusCode).toBe(200);
		expect(response.headers["content-type"]).toMatch(/application\/json/);
		expect(response.headers["cache-control"]).toBe("max-age=86400");
		expect(response.headers["access-control-allow-origin"]).toBe("*");
		expect(response.json<Menu[]>().length).toBeGreaterThan(5000);
	});

	describe("single menu", () => {
		it("should work", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/menus/sodexo.2ae66740-672e-4183-ab2d-ac1e00b66a5f",
			});

			expect(response.statusCode).toBe(200);
			expect(response.headers["content-type"]).toMatch(/application\/json/);
			expect(response.headers["cache-control"]).toBe("max-age=86400");
		});

		it("should return 404, not 500", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/menus/sodexo.0",
			});

			expect(response.statusCode).toBe(404);
			expect(response.headers["cache-control"]).toBeUndefined();
		});
	});

	describe("menu days", () => {
		it("should not accept invalid timestamps", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/menus/skolmaten.85957002/days",
				query: {
					first: "invalid-iso8601",
				},
			});

			expect(response.statusCode).toBe(422);
			expect(response.headers["cache-control"]).toBeUndefined();
		});

		it("should assert the timestamps are in order", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/menus/skolmaten.85957002/days",
				query: {
					first: "2021-01-01",
					last: "2020-01-01",
				},
			});

			expect(response.statusCode).toBe(400);
			expect(response.headers["cache-control"]).toBeUndefined();
		});

		it("should work without any parameters", async () => {
			const response = await app.inject({
				method: "GET",
				url: "/menus/skolmaten.85957002/days",
			});

			expect(response.statusCode).toBe(200);
			expect(response.headers["content-type"]).toMatch(/application\/json/);
			expect(response.headers["cache-control"]).toBe("max-age=86400");
		});
	});
});
