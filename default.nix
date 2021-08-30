with import<nixpkgs>{};
rustPlatform.buildRustPackage rec {
  pname = "rcol";
  version = "0.1";

  src = ./.;

  cargoSha256 = "1ld1yf6hflr1klzvbz6gi2r2mp5b2rzjaq8br7lqxpczvjyc4vid";

  meta = with stdenv.lib; {
    description = "Automatic log colorizer.";
    homepage = "https://github.com/nthorne/rcol";
    license = licenses.mit;
    maintainers = [ maintainers.nthorne ];
  };
}
