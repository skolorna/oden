# Skolmaten API

[skolmaten.se has an API](https://skolmaten.se/about/api/), but unfortunately it is private. Not anymore, though. The only authentication required is an application ID (that you cannot get anywhere), but my theory is that the iOS and Android apps have application IDs available. I just need to extract them ...

Update: By using [this handy utility](https://github.com/alessandrodd/apk_api_key_extractor) to extract all the API keys from the APK, I was able to obtain the API key used in [the Android app](https://play.google.com/store/apps/details?id=se.yo.android.skolmat&hl=sv), which makes calls from this client indistinguishable from "real" ones.

They key is: `j44i0zuqo8izmlwg5blh`.
