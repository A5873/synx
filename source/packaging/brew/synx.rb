class Synx < Formula
  desc "Universal syntax validator and linter dispatcher"
  homepage "https://github.com/A5873/synx"
  version "0.2.1"
  license "MIT"

  depends_on "rust" => :build
  depends_on "pkg-config" => :build
  depends_on "openssl"

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/synx"
  end

  test do
    system "#{bin}/synx", "--version"
  end
end
