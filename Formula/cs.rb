class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.2.5/cs-darwin-amd64"
  sha256 "85caaf0d5785cafd53e8eece8dbebdd16351c86547a98b40aea304885269dd80"
  version "0.2.5"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
