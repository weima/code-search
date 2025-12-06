class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.2.3/cs-darwin-amd64"
  sha256 "ab215ee934e8e5e101b485ef7a07003add91e3062a2c9b555cb7b5a057bcff31"
  version "0.2.3"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
