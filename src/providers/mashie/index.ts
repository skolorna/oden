import { ProviderImplementation } from "../types";
import { getMashieMenuGetter } from "./menu";
import { getMashieSchoolLister } from "./schools";
import { MashieGenerator } from "./types";

export const generateMashieImplementation: MashieGenerator<ProviderImplementation> = (baseUrl) => {
	return {
		getMenu: getMashieMenuGetter(baseUrl),
		listSchools: getMashieSchoolLister(baseUrl),
	};
};
