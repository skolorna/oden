import { DistrictsResponse, ProvincesResponse, SchoolsResponse } from "./types";
import School from "../../types/school";
import performSkolmatenRequest from "./request";

const crawlSchools = async (): Promise<School[]> => {
  const { provinces } = await performSkolmatenRequest<ProvincesResponse>("/provinces");

  const schools: School[][][] = await Promise.all(provinces.map(async (province) => {
    const { districts } = await performSkolmatenRequest<DistrictsResponse>(`/districts?province=${province.id}`);

    return Promise.all(districts.map(async (district) => {
      const { schools } = await performSkolmatenRequest<SchoolsResponse>(`/schools?district=${district.id}`);

      return schools.map(({ id, name }) => ({
        id: id.toString(),
        name,
        district: district.name,
        province: province.name,
      }));
    }));
  }));

  return schools.flat(2);
}

export default crawlSchools;
