import { DistrictsResponse, ProvincesResponse, SchoolsResponse } from "./types";
import performSkolmatenRequest from "./request";
import { GetSchools } from "../types";
import { School } from "../../types";

export const getSkolmatenSchools: GetSchools = async () => {
	const { provinces } = await performSkolmatenRequest<ProvincesResponse>("/provinces");

	const school3d: School[][][] = await Promise.all(
		provinces.map(async (province) => {
			const { districts } = await performSkolmatenRequest<DistrictsResponse>(`/districts?province=${province.id}`);

			return Promise.all(
				districts.map(async (district) => {
					const { schools } = await performSkolmatenRequest<SchoolsResponse>(`/schools?district=${district.id}`);

					return schools.map(({ id, name }) => ({
						id: id.toString(),
						name,
						district: district.name,
						province: province.name,
					}));
				}),
			);
		}),
	);

	return school3d.flat(2);
};
