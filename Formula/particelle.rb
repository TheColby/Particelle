class Particelle < Formula
  desc "Algorithmic granular synthesis engine with microtonal and multichannel support"
  homepage "https://github.com/TheColby/Particelle"
  version "0.5.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TheColby/Particelle/releases/download/v0.5.0/particelle_v0.5.0_aarch64-apple-darwin.tar.gz"
      sha256 "a35a587e00d744ff6b13e78aba002cb962fbe7fa8e2855dbf1a390fc8892733b"
    else
      url "https://github.com/TheColby/Particelle/releases/download/v0.5.0/particelle_v0.5.0_x86_64-apple-darwin.tar.gz"
      sha256 "5bf64225897b3ea34775533eb2d00dde3e78f2c9f5a76392ccae51614ca73513"
    end
  end

  on_linux do
    url "https://github.com/TheColby/Particelle/releases/download/v0.5.0/particelle_v0.5.0_x86_64-unknown-linux-gnu.tar.gz"
    sha256 "4fdd0744b7b7282ae56628757c0c968a7d7df4594cf0b5ec8fc2004f96f247e0"
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
