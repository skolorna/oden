fn main() {
    pkg_config::Config::new();

    let src = ["src/shoco.c"];
    let mut builder = cc::Build::new();
    let build = builder.files(src.iter()).include("src");

    build.compile("shoco");
}
