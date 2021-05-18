export interface SkolmatenObject {
  id: number;
  name: string;
}

export interface ProvincesResponse {
  provinces: SkolmatenObject[];
}

export interface DistrictsResponse {
  districts: SkolmatenObject[];
}

export interface SchoolsResponse {
  schools: SkolmatenObject[];
}
