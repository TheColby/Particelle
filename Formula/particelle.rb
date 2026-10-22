class Particelle < Formula
  desc "Algorithmic granular synthesis engine with microtonal and multichannel support"
  homepage "https://github.com/TheColby/Particelle"
  version "0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TheColby/Particelle/releases/download/v0.4.0/particelle_v0.4.0_aarch64-apple-darwin.tar.gz"
      sha256 "9a4078bdcb2908cadddbd5e13fba75485672814e200cc5ddb96827fc69894ff5"
    else
      url "https://github.com/TheColby/Particelle/releases/download/v0.4.0/particelle_v0.4.0_x86_64-apple-darwin.tar.gz"
      sha256 "1d3274495b92ed2333053af62267838f4b08edd864569c723999753e2485c1a2"
    end
  end

  on_linux do
    url "https://github.com/TheColby/Particelle/releases/download/v0.4.0/particelle_v0.4.0_x86_64-unknown-linux-gnu.tar.gz"
    sha256 "dbd08611edc8bca42fcf429e436fd80380fe801f7757f18b7783def7915ee364"
  end

  def install
    binary = Dir["bin/particelle", "*/bin/particelle"].first
    odie "Expected particelle binary in release tarball, but none was found." if binary.nil?

    bin.install binary => "particelle"
    bin.install_symlink "particelle" => "ptc"
  end

  test do
    assert_match "particelle", shell_output("#{bin}/particelle --version")
  end
end
