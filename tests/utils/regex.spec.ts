import { LOCALHOST_HTTP_REGEX, SKOLORNA_HTTP_REGEX } from "../../src/utils/regex";

type RegExpTestTable = {
	name: string;
	regex: RegExp;
	positives: string[];
	negatives: string[];
}[];

const tests: RegExpTestTable = [
	{
		name: "localhost",
		regex: LOCALHOST_HTTP_REGEX,
		positives: ["http://localhost:8080", "http://localhost", "http://localhost:3000", "https://localhost:8443"],
		negatives: ["https://google.com", "http://localhost.com", "http://localhost/path", "ssh://localhost:22"],
	},
	{
		name: "skolorna",
		regex: SKOLORNA_HTTP_REGEX,
		positives: ["https://skolorna.com", "http://skolorna.com", "http://mat.skolorna.com", "http://mat.i.skolorna.com"],
		negatives: ["ssh://skolorna.com", "http://fakeskolorna.com", "http://skolorna.com:22", "http://skolorna.com.fake"],
	},
];

describe("regex tests", () => {
	tests.forEach(({ name, regex, positives, negatives }) => {
		test(`${name} regex`, () => {
			positives.forEach((text) => {
				expect(text).toMatch(regex);
			});

			negatives.forEach((text) => {
				expect(text).not.toMatch(regex);
			});
		});
	});
});
