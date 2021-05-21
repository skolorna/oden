import { Static, Type } from "@sinclair/typebox";

export const QuerySchoolOptions = Type.Object({
	schoolId: Type.String(),
});

export type QuerySchoolOptionsType = Static<typeof QuerySchoolOptions>;
