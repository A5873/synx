class Synx < Formula
  desc "CLI-first universal syntax validator and linter dispatcher"
  homepage "https://github.com/A5873/synx"
  url "https://github.com/A5873/synx/archive/v0.2.0.tar.gz"
  sha256 "SKIP" # We'll update this with actual SHA after release
  license "MIT"

  depends_on "rust" => :build
  depends_on "gcc" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/synx", "--version"
  end
end
