import { ProviderImplementation } from "../types";
import { getMashieMenuGetter } from "./menu";
import { getMashieSchoolLister, getMashieSchoolQuerier } from "./schools";
import { MashieGenerator } from "./types";

export const generateMashieImplementation: MashieGenerator<ProviderImplementation> = (baseUrl) => {
	return {
		getMenu: getMashieMenuGetter(baseUrl),
		listSchools: getMashieSchoolLister(baseUrl),
		querySchool: getMashieSchoolQuerier(baseUrl),
	};
};
