import fetch from "node-fetch";
import { GetSchools } from "../types";
import { GetSchoolsResponse } from "./types";

export const getMashieSchools: GetSchools = async () => {
  const data: GetSchoolsResponse = await fetch("https://sodexo.mashie.com/public/app/internal/execute-query?country=se", {
    method: "POST",
  }).then((res) => res.json());

  return data.map(({
    id,
    title,
  }) => ({
    id,
    name: title,
  }));
}
