class Particelle < Formula
  desc "Algorithmic granular synthesis engine with microtonal and multichannel support"
  homepage "https://github.com/TheColby/Particelle"
  version "0.3.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TheColby/Particelle/releases/download/v0.3.1/particelle_v0.3.1_aarch64-apple-darwin.tar.gz"
      sha256 "640ad626aa07c7438f6c701c8734ef5f682e4c7b51452e9a97b2ad2f4caf391d"
    else
      url "https://github.com/TheColby/Particelle/releases/download/v0.3.1/particelle_v0.3.1_x86_64-apple-darwin.tar.gz"
      sha256 "b10d2e7864ae02628aa18c57f558399622dd6d3be55109b7fa1156a87152e4d2"
    end
  end

  on_linux do
    url "https://github.com/TheColby/Particelle/releases/download/v0.3.1/particelle_v0.3.1_x86_64-unknown-linux-gnu.tar.gz"
    sha256 "0900b82dbc5df16cede3196278f9e56a40aa101c0b2896853058ea4e333654d9"
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
