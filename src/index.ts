import { getSkolmatenSchools } from "./upstream/skolmaten/crawler";

getSkolmatenSchools().then((schools) => {
  console.log(schools);
})
