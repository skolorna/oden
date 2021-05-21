export interface MashieSchool {
	id: string;
	title: string;
	url: string;
}

export type GetSchoolsResponse = MashieSchool[];

/**
 * A function that generates a provider-specific implementation of something.
 */
export type MashieGenerator<T> = (baseUrl: string) => T;

export type QueryMashieSchool = (id: string) => Promise<MashieSchool>;
