class Particelle < Formula
  desc "Algorithmic granular synthesis engine with microtonal and multichannel support"
  homepage "https://github.com/TheColby/Particelle"
  version "0.2.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TheColby/Particelle/releases/download/v0.2.3/particelle_v0.2.3_aarch64-apple-darwin.tar.gz"
      sha256 "8771796a8996eae304599b3197e7ce147af13323e2e6205fa1cb1f8b72294871"
    else
      url "https://github.com/TheColby/Particelle/releases/download/v0.2.3/particelle_v0.2.3_x86_64-apple-darwin.tar.gz"
      sha256 "1bda183499e84d402374f035b242e781d75a787a7e1f74180dd3cfcacf45b588"
    end
  end

  on_linux do
    url "https://github.com/TheColby/Particelle/releases/download/v0.2.3/particelle_v0.2.3_x86_64-unknown-linux-gnu.tar.gz"
    sha256 "882476197a892d2e474c02235d5cb9b29c94922242b50beaf688ed072746bcc8"
  end

  def install
    binary = Dir["*/bin/particelle"].first
    odie "Expected particelle binary in release tarball, but none was found." if binary.nil?

    bin.install binary => "particelle"
    bin.install_symlink "particelle" => "ptc"
  end

  test do
    assert_match "particelle", shell_output("#{bin}/particelle --version")
  end
end
