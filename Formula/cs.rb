class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.3.3/cs-darwin-amd64"
  sha256 "5b47654dd7e8b1dd3042f80461fa07a5e9199ff901a902537e783e615de5d506"
  version "0.3.3"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
