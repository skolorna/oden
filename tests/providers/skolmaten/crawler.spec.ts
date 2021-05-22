import { getSkolmatenSchools, validateSchoolName } from "../../../src/providers/skolmaten/crawler";

test("school name validation", () => {
	expect(validateSchoolName("Information!")).toBeFalsy();
	expect(validateSchoolName("Södra Latin")).toBeTruthy();
});

test("crawler", async () => {
	jest.setTimeout(20000);

	const schools = await getSkolmatenSchools();

	expect(schools.length).toBeGreaterThan(1000);
});
