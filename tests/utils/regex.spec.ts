import { LOCALHOST_HTTP_REGEX } from "../../src/utils/regex";

describe("regex tests", () => {
	test("localhost regex", () => {
		const positives = ["http://localhost:8080", "http://localhost", "http://localhost:3000", "https://localhost:8443"];

		positives.forEach((text) => {
			expect(text).toMatch(LOCALHOST_HTTP_REGEX);
		});

		const negatives = ["https://google.com", "http://localhost.com", "http://localhost/path", "ssh://localhost:22"];

		negatives.forEach((text) => {
			expect(text).not.toMatch(LOCALHOST_HTTP_REGEX);
		});
	});
});
