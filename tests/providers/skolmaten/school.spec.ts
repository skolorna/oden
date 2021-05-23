import { NotFound } from "http-errors";
import { querySkolmatenSchool } from "../../../src/providers/skolmaten/school";

test("skolmaten school", async () => {
	const school = await querySkolmatenSchool("85957002");

	expect(school.name).toMatch(/P\s?A Fogelström/i);

	await expect(querySkolmatenSchool("a")).rejects.toThrow();
	await expect(querySkolmatenSchool("123")).rejects.toThrowError(NotFound);
});
