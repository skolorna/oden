import { NotFound } from "http-errors";
import { getMashieSchoolLister, getRawMashieSchoolQuerier } from "../../../src/providers/mashie/schools";

test("list schools", async () => {
	const listMashieSchools = getMashieSchoolLister("https://sodexo.mashie.com");

	const schools = await listMashieSchools();

	expect(schools.length).toBeGreaterThan(100);
});

describe("querying schools", () => {
	const queryMashieSchool = getRawMashieSchoolQuerier("https://sodexo.mashie.com");

	it("should throw an error if no school is found", async () => {
		await expect(queryMashieSchool("invalid-id-that-should-not-be-used")).rejects.toThrowError(NotFound);
	});

	it("should work as expected", async () => {
		const school = await queryMashieSchool("b4639689-60f2-4a19-a2dc-abe500a08e45");

		expect(school.title).toMatch(/Norra Real/i);
	});
});
