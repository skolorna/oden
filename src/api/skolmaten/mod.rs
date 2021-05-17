static API_TOKEN: &str = "j44i0zuqo8izmlwg5blh";

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn api_key_is_valid() {
      fn get_status_code(token: &str) -> u16 {
          match ureq::get("https://skolmaten.se/api/3/districts?province=5758661280399360")
              .set("Client", token)
              .call()
          {
              Ok(response) => response.status(),
              Err(ureq::Error::Status(status, _)) => status,
              Err(err) => panic!("{}", err),
          }
      }

      assert_eq!(get_status_code(API_TOKEN), 200);
  }
}
