import performSkolmatenRequest from "../../../../src/menus/providers/skolmaten/request";
import { DistrictsResponse } from "../../../../src/menus/providers/skolmaten/types";

test("skolmaten api client", async () => {
	const { districts } = await performSkolmatenRequest<DistrictsResponse>("/districts/?province=5662940552232960");

	expect(districts.length).toBeGreaterThan(10);
});
