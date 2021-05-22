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

export interface DetailedStation extends DetailedSkolmatenObject {
	imageURL: string;
	district: DetailedDistrict;
	location?: {
		latitude: number;
		longitude: number;
	};
}

export interface ProvincesResponse {
	provinces: SkolmatenObject[];
}

export interface DistrictsResponse {
	districts: SkolmatenObject[];
}

export interface SkolmatenStationsResponse {
	stations: SkolmatenObject[];
}

export interface Bulletin {
	text: string;
}

export interface SkolmatenMeal {
	value: string;
	attributes: number[];
}

export interface SkolmatenDay {
	year: number;
	month: number;
	day: number;
	meals?: SkolmatenMeal[];
	reason?: string;
}

export interface SkolmatenWeek {
	year: number;
	weekOfYear: number;
	days: SkolmatenDay[];
}

export interface MenuResponse {
	menu: {
		isFeedbackAllowed: boolean;
		weeks: SkolmatenWeek[];
		station: DetailedStation;
		id: number;
		bulletins: Bulletin[];
	};
}

export interface SkolmatenTimeRange {
	year: number;
	weekOfYear: number;
	count: number;
}
