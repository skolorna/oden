import { NotFound } from "http-errors";
import fetch from "node-fetch";
import { URL } from "url";
import { ListSchools, QuerySchool } from "../types";
import { GetSchoolsResponse, MashieGenerator, QueryMashieSchool } from "./types";

const getSchoolFetcher: MashieGenerator<() => Promise<GetSchoolsResponse>> = (baseUrl) => {
	return async () => {
		const data: GetSchoolsResponse = await fetch(new URL("/public/app/internal/execute-query?country=se", baseUrl), {
			method: "POST",
		}).then((res) => res.json());

		return data;
	};
};

export const getMashieSchoolLister: MashieGenerator<ListSchools> = (baseUrl) => {
	const fetchSchools = getSchoolFetcher(baseUrl);

	return async () => {
		const schools = await fetchSchools();

		return schools.map(({ id, title }) => ({
			id,
			name: title,
		}));
	};
};

export const getRawMashieSchoolQuerier: MashieGenerator<QueryMashieSchool> = (baseUrl) => {
	const fetchSchools = getSchoolFetcher(baseUrl);

	return async (id) => {
		const schools = await fetchSchools();

		const school = schools.find(({ id: schoolID }) => schoolID === id);

		if (!school) {
			throw new NotFound(`school with id \`${id}\` not found`);
		}

		return school;
	};
};

export const getMashieSchoolQuerier: MashieGenerator<QuerySchool> = (baseUrl) => {
	const querySchool = getRawMashieSchoolQuerier(baseUrl);

	return async (queryId) => {
		const { title, id } = await querySchool(queryId);

		return {
			name: title,
			id,
		};
	};
};
