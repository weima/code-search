class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.15/cs-darwin-amd64"
  sha256 "c987de51e8a255202c11e678bdff36f210ed96aee6deae8a5a69813ec8bce084"
  version "0.1.15"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
