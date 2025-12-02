class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.7/cs-darwin-amd64"
  sha256 "93c74100e7c5b4435a0ca06aea822a0a98f76df0a24894ded8e4e8a8a7d7c9bb"
  version "0.1.7"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
