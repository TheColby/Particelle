class Particelle < Formula
  desc "Algorithmic granular synthesis engine with microtonal and multichannel support"
  homepage "https://github.com/TheColby/Particelle"
  version "0.3.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TheColby/Particelle/releases/download/v0.3.2/particelle_v0.3.2_aarch64-apple-darwin.tar.gz"
      sha256 "cb212f83bab3973683cc3580b5461cab92bf8fffeb2cc25ee63f211e28a81028"
    else
      url "https://github.com/TheColby/Particelle/releases/download/v0.3.2/particelle_v0.3.2_x86_64-apple-darwin.tar.gz"
      sha256 "66b126af92a07dda2d5f01546f2547583b25b5995928aa925f4be93a30ad12d0"
    end
  end

  on_linux do
    url "https://github.com/TheColby/Particelle/releases/download/v0.3.2/particelle_v0.3.2_x86_64-unknown-linux-gnu.tar.gz"
    sha256 "ada56362633ff275fa4fa570a4885ac215612c31c0f906c2faf74c520c69fcc9"
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
