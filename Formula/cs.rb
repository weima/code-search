class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.5/cs-darwin-amd64"
  sha256 "c088db9f4fbfe73fa75140f6a9378ccf7b2ac403b6ff9a709b5c39598a9715f4"
  version "0.1.5"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
