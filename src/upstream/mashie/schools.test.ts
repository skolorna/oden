import { getMashieSchools } from "./schools";

test("mashie schools", async () => {
  const schools = await getMashieSchools();

  expect(schools.length).toBeGreaterThan(100);
});
