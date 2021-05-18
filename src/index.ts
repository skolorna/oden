import crawlSchools from "./upstream/skolmaten/crawlSchools";

crawlSchools().then((schools) => {
  console.log(schools);
})
