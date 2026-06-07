{ lib, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "rstart";
  version = "TEMPLATE_VERSION";

  src = ../..;

  cargoSha256 = lib.fakeSha256; # Users will replace this with actual hash

  meta = with lib; {
    description = "A project template for creating unified local terminal utilities in Rust";
    homepage = "https://github.com/local76/rStartrt";
    license = licenses.mit;
    maintainers = [ "UberMetroid" ];
  };
}
