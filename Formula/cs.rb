class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.12/cs-darwin-amd64"
  sha256 "d065da712864bc423c9dd381ae146e6a822fd9bb0bdb1081f67c26e99344fbe2"
  version "0.1.12"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
