export interface SkolmatenObject {
	id: number;
	name: string;
}

export interface DetailedSkolmatenObject extends SkolmatenObject {
	URLName: string;
}

export interface DetailedDistrict extends DetailedSkolmatenObject {
	province: DetailedSkolmatenObject;
}

export interface DetailedSchool extends DetailedSkolmatenObject {
	imageURL: string;
	district: DetailedDistrict;
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

export interface Bulletin {
	text: string;
}

export interface SkolmatenMeal {
	value: string;
	attributes: number[];
}

export interface SkolmatenDay {
	date: number;
	meals?: SkolmatenMeal[];
	reason?: string;
}

export interface SkolmatenWeek {
	year: number;
	number: number;
	days: SkolmatenDay[];
}

export interface MenuResponse {
	weeks: SkolmatenWeek[];
	school: DetailedSchool;
	bulletins: Bulletin[];
}
