import performSkolmatenRequest from "./request";
import { DistrictsResponse, ProvincesResponse } from "./types";

test("skolmaten api client", async () => {
  const { districts } = await performSkolmatenRequest<DistrictsResponse>("/districts/?province=5662940552232960");

  expect(districts.length).toBeGreaterThan(10);
});
