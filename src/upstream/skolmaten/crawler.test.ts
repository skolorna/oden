import { getSkolmatenSchools } from "./crawler";

test("crawler", async () => {
  jest.setTimeout(20000);

  const schools = await getSkolmatenSchools();

  expect(schools.length).toBeGreaterThan(1000);
});
