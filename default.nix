with import<nixpkgs>{};
rustPlatform.buildRustPackage rec {
  pname = "rcol";
  version = "0.1";

  src = ./.;

  cargoSha256 = "1pqc76ffcyk393vic72k1cy2nmrxgz4icmrxar1a935ry9jzci4n";

  meta = with stdenv.lib; {
    description = "Automatic log colorizer.";
    homepage = "https://github.com/nthorne/rcol";
    license = licenses.mit;
    maintainers = [ maintainers.nthorne ];
  };
}
