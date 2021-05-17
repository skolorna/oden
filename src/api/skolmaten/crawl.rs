use super::{super::School, API_TOKEN};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct SkolmatenObject {
    pub id: u64,
    pub name: String,
}

type Province = SkolmatenObject;

#[derive(Deserialize, Debug)]
struct ProvincesResponse {
    pub provinces: Vec<Province>,
}

type District = SkolmatenObject;

#[derive(Deserialize, Debug)]
struct DistrictsResponse {
    districts: Vec<District>,
}

#[derive(Deserialize, Debug)]
struct SchoolsResponse {
    schools: Vec<SkolmatenObject>,
}

impl Province {
    pub fn get_districts(&self) -> Result<Vec<District>, ureq::Error> {
        let url = format!(
            "https://skolmaten.se/api/3/districts?province={}",
            self.id.to_string()
        );

        let res: DistrictsResponse = ureq::get(&url)
            .set("Client", API_TOKEN)
            .call()?
            .into_json()?;

        Ok(res.districts)
    }
}

impl District {
    pub fn get_schools(&self) -> Result<Vec<School>, ureq::Error> {
        let url = format!(
            "https://skolmaten.se/api/3/schools?district={}",
            self.id.to_string()
        );

        let res: SchoolsResponse = ureq::get(&url)
            .set("Client", API_TOKEN)
            .call()?
            .into_json()?;

        let schools = res
            .schools
            .into_iter()
            .map(|school| School {
                id: school.id.to_string(),
                name: school.name,
            })
            .collect();

        Ok(schools)
    }
}

fn get_provinces() -> Result<Vec<SkolmatenObject>, ureq::Error> {
    let res: ProvincesResponse = ureq::get("https://skolmaten.se/api/3/provinces")
        .set("Client", API_TOKEN)
        .call()?
        .into_json()?;

    Ok(res.provinces)
}

pub fn crawl_schools() -> Result<Vec<School>, ureq::Error> {
    let provinces = get_provinces()?;

    let schools = provinces
        .into_iter()
        .flat_map(|province| {
            let districts = province.get_districts().unwrap();

            let province_schools = districts
                .into_iter()
                .flat_map(|district| district.get_schools().unwrap());

            province_schools
        })
        .collect();

    Ok(schools)
}
